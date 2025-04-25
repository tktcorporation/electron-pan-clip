#![cfg(target_os = "windows")]

use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::iter::once;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::{
  Foundation::{GetLastError, HWND},
  System::{
    DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
    Memory::{GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
  },
  UI::Shell::{DROPFILES, CF_HDROP},
};

// ワイド文字列（UTF-16）に変換し、NULL終端を追加するヘルパー関数
fn to_wide_null(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().chain(once(0)).collect()
}

pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  // 1. パスリストをワイド文字列（UTF-16）に変換し、ダブルNULL終端形式にする
  let mut wide_paths: Vec<u16> = paths
    .iter()
    .map(|p| to_wide_null(p)) // 各パスを NULL 終端 UTF-16 に変換
    .flatten() // 平坦化して連結
    .collect();
  wide_paths.push(0); // リストの最後に NULL を追加してダブルNULL終端にする

  // 2. DROPFILES 構造体とパスリストを格納するためのメモリサイズを計算
  let dropfiles_size = size_of::<DROPFILES>();
  let paths_size = wide_paths.len() * size_of::<u16>();
  let total_size = dropfiles_size + paths_size;

  unsafe {
    // 3. グローバルメモリを確保
    // CF_HDROP は GMEM_MOVEABLE である必要がある
    let h_global = GlobalAlloc(GMEM_MOVEABLE, total_size);
    if h_global == 0 as windows_sys::Win32::Foundation::HANDLE {
      let err = GetLastError();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to allocate global memory: {}", err),
      ));
    }

    // 4. メモリをロックしてポインタを取得
    let buffer_ptr = GlobalLock(h_global) as *mut u8;
    if buffer_ptr.is_null() {
      let err = GetLastError();
      GlobalFree(h_global);
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to lock global memory: {}", err),
      ));
    }

    // 5. DROPFILES 構造体をメモリに書き込む
    let dropfiles_ptr = buffer_ptr as *mut DROPFILES;
    *dropfiles_ptr = DROPFILES {
      pFiles: dropfiles_size as u32, // パスリストへのオフセット
      pt: zeroed(),                  // 使わないのでゼロ初期化
      fNC: false.into(),             // 非クライアント領域座標ではない
      fWide: true.into(),            // ワイド文字（UTF-16）を使用
    };

    // 6. パスリスト（ダブルNULL終端）を DROPFILES 構造体の直後に書き込む
    let paths_ptr = buffer_ptr.add(dropfiles_size);
    std::ptr::copy_nonoverlapping(wide_paths.as_ptr() as *const u8, paths_ptr, paths_size);

    // 7. メモリをアンロック
    GlobalUnlock(h_global);

    // 8. クリップボードを開く (所有者を指定しない場合は NULL)
    if OpenClipboard(0 as HWND) == 0 {
      let err = GetLastError();
      GlobalFree(h_global); // 失敗したら確保したメモリを解放
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {}", err),
      ));
    }

    // 9. クリップボードを空にする
    if EmptyClipboard() == 0 {
      let err = GetLastError();
      CloseClipboard();
      GlobalFree(h_global);
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to empty clipboard: {}", err),
      ));
    }

    // 10. CF_HDROP 形式でデータをクリップボードに設定
    // SetClipboardData が成功すると、OS がメモリの所有権を持つため、GlobalFree を呼んではいけない
    if SetClipboardData(CF_HDROP as u32, h_global as isize) == 0 {
      let err = GetLastError();
      CloseClipboard();
      GlobalFree(h_global); // 失敗したので解放
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to set clipboard data: {}", err),
      ));
    }

    // 11. クリップボードを閉じる
    if CloseClipboard() == 0 {
      // この時点ではデータは設定されているが、閉じるのに失敗した
      // エラーとして報告するべきか？ 일단 ここでは警告としておく
      eprintln!("Warning: Failed to close clipboard: {}", GetLastError());
    }
  } // unsafe ブロック終了

  println!("Copied files to clipboard on Windows: {:?}", paths);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::temp_dir;
  use std::fs::File;
  use std::path::Path;

  // UTF-16エンコードのテスト
  #[test]
  fn test_to_wide_null() {
    let test_str = "Hello, Windows!";
    let wide = to_wide_null(test_str);

    // NULL終端文字を含むため、長さは元の文字列+1になる
    assert_eq!(wide.len(), test_str.len() + 1);

    // 最後の文字は NULL (0) であることを確認
    assert_eq!(wide[wide.len() - 1], 0);

    // 各文字がUTF-16エンコードされていることを確認
    for (i, c) in test_str.chars().enumerate() {
      assert_eq!(wide[i], c as u16);
    }
  }

  // パスの変換テスト
  #[test]
  fn test_path_conversion() {
    // Windowsパスのテスト
    let test_paths = vec![
      "C:\\Windows\\System32".to_string(),
      "D:\\Documents\\file.txt".to_string(),
    ];

    // パスをワイド文字列に変換
    let mut wide_paths: Vec<u16> = test_paths
      .iter()
      .map(|p| to_wide_null(p))
      .flatten()
      .collect();
    wide_paths.push(0); // ダブルNULL終端

    // 各パスのNULL区切りを確認
    let mut null_count = 0;
    for c in wide_paths.iter() {
      if *c == 0 {
        null_count += 1;
      }
    }

    // パスの数 + ダブルNULL用の追加1個 = NULL文字の総数
    assert_eq!(null_count, test_paths.len() + 1);
  }

  // 実際のクリップボード操作テスト
  #[test]
  #[ignore]
  fn test_copy_real_files() {
    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_windows_clipboard.txt");

    // ファイルを作成
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // パスを文字列に変換
    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = copy_files_to_clipboard(&[path_str]);
    assert!(result.is_ok(), "Failed to copy files: {:?}", result);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 空のパスリストのテスト
  // Windowsの実装では空のリストでもメモリ確保などは行うが、
  // エラーにはならない。ただし実用上は空リスト前にチェックする方が良い
  #[test]
  fn test_empty_paths() {
    let result = copy_files_to_clipboard(&[]);
    // この実装では空リストでもエラーにはならない
    // 注: lib.rs側で空チェックを行っているため、通常は到達しない
    assert!(result.is_ok());
  }
}
