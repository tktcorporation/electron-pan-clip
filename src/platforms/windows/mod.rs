use std::ffi::OsString;
use std::io;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use windows::Win32::Foundation::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::UI::Shell::DROPFILES;

// クリップボードフォーマット定数
const CF_HDROP: u32 = 15;

#[allow(dead_code)]
/// WindowsのCF_HDROP形式でファイルをクリップボードにコピーする
pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), io::Error> {
  // ファイルパスをワイド文字列に変換して、必要なサイズを計算
  let mut total_size = std::mem::size_of::<DROPFILES>();
  let wide_paths: Vec<Vec<u16>> = paths
    .iter()
    .map(|s| OsString::from(s).encode_wide().chain(once(0)).collect())
    .collect();

  for wide_path in &wide_paths {
    total_size += wide_path.len() * std::mem::size_of::<u16>();
  }

  // 終端のダブルNULL用に追加
  total_size += std::mem::size_of::<u16>();

  unsafe {
    // グローバルメモリを確保
    let h_global = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size).unwrap();
    if h_global.is_invalid() {
      return Err(io::Error::last_os_error());
    }

    // メモリをロックしてDROPFILES構造体を作成
    let mem_ptr = GlobalLock(h_global);
    if mem_ptr.is_null() {
      let _ = GlobalFree(h_global);
      return Err(io::Error::last_os_error());
    }

    // DROPFILES構造体を初期化
    let drop_files_ptr = mem_ptr as *mut DROPFILES;
    (*drop_files_ptr).pFiles = std::mem::size_of::<DROPFILES>() as u32;
    (*drop_files_ptr).fWide = BOOL(1); // UNICODE文字列を使用する

    // ファイルパスの配列を構築
    let mut dest_ptr = (mem_ptr as usize + std::mem::size_of::<DROPFILES>()) as *mut u16;
    for wide_path in &wide_paths {
      ptr::copy_nonoverlapping(wide_path.as_ptr(), dest_ptr, wide_path.len());
      dest_ptr = dest_ptr.add(wide_path.len());
    }
    // 最後のNULLターミネータを追加して配列を終了
    *dest_ptr = 0;

    // メモリをアンロック
    GlobalUnlock(h_global).ok();

    // クリップボードを開く
    if OpenClipboard(HWND(0)).is_err() {
      let _ = GlobalFree(h_global);
      return Err(io::Error::last_os_error());
    }

    // クリップボードをクリアしてデータをセット
    EmptyClipboard().ok();
    let h_result = SetClipboardData(CF_HDROP, HANDLE(h_global.0 as isize));
    CloseClipboard().ok();

    if h_result.is_err() {
      let _ = GlobalFree(h_global);
      return Err(io::Error::last_os_error());
    }

    // 成功した場合、hGlobalの所有権はシステムに移るので解放しない
    Ok(())
  }
}
