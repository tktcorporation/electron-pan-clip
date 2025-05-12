// Linux向けのクリップボード操作実装

use std::io::{Error, ErrorKind};
use std::process::Command;
use std::fs;

// xclip コマンドを使用してファイルパスをクリップボードにコピーする
pub fn write_clipboard_file_paths(paths: &[String]) -> Result<(), Error> {
  // URIに変換
  let mut uri_paths = Vec::new();
  let mut errors = Vec::new();

  // パスがある場合のみURIへの変換を行う
  if !paths.is_empty() {
    // 各パスをURIに変換
    for path in paths {
      let canonical_path = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
          errors.push(format!("Failed to canonicalize path {}: {}", path, e));
          continue;
        }
      };

      // file:// URIを作成
      let uri = format!("file://{}", canonical_path.display());
      uri_paths.push(uri);
    }

    // 無効なパスが一つでもあればエラー
    if !errors.is_empty() {
      let error_message = format!(
        "Some paths could not be processed: {}",
        errors.join("; ")
      );
      return Err(Error::new(ErrorKind::InvalidInput, error_message));
    }
  }

  // URIをタブ区切りでつなげる（GNOMEの標準フォーマット）
  let joined_uris = uri_paths.join("\n");

  // xclipコマンドでクリップボードに書き込む
  let mut command = Command::new("xclip");
  command
    .arg("-selection")
    .arg("clipboard")
    .arg("-t")
    .arg("text/uri-list");

  // コマンドにデータをパイプして実行
  let status = command
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::null())
    .spawn()
    .and_then(|mut child| {
      use std::io::Write;
      if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(joined_uris.as_bytes())?;
      }
      child.wait()
    });

  match status {
    Ok(exit_status) if exit_status.success() => {
      if paths.is_empty() {
        println!("Cleared clipboard data (empty file list)");
      } else {
        println!("Copied {} files to clipboard on Linux", paths.len());
      }
      Ok(())
    },
    Ok(exit_status) => Err(Error::new(
      ErrorKind::Other,
      format!(
        "xclip command failed with exit code: {:?}",
        exit_status.code()
      ),
    )),
    Err(e) => Err(Error::new(
      ErrorKind::Other,
      format!("Failed to execute xclip command: {}", e),
    )),
  }
}

// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  // xclipコマンドでクリップボードからテキストを読み取る
  let output = Command::new("xclip")
    .arg("-selection")
    .arg("clipboard")
    .arg("-o")
    .output()?;

  if output.status.success() {
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.is_empty() {
      Err(Error::new(ErrorKind::Other, "No text content in clipboard"))
    } else {
      Ok(text)
    }
  } else {
    let error = String::from_utf8_lossy(&output.stderr).into_owned();
    Err(Error::new(
      ErrorKind::Other,
      format!("Failed to read clipboard: {}", error),
    ))
  }
}

// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  // xclipコマンドでクリップボードからデータを読み取る
  let output = Command::new("xclip")
    .arg("-selection")
    .arg("clipboard")
    .arg("-o")
    .output()?;

  if output.status.success() {
    if output.stdout.is_empty() {
      Err(Error::new(ErrorKind::Other, "No data in clipboard"))
    } else {
      Ok(output.stdout)
    }
  } else {
    let error = String::from_utf8_lossy(&output.stderr).into_owned();
    Err(Error::new(
      ErrorKind::Other,
      format!("Failed to read clipboard raw data: {}", error),
    ))
  }
}

// クリップボードからファイルパスを読み取る
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  // xclipコマンドでクリップボードからURI-listを読み取る
  let output = Command::new("xclip")
    .arg("-selection")
    .arg("clipboard")
    .arg("-o")
    .arg("-t")
    .arg("text/uri-list")
    .output()?;

  if output.status.success() {
    let content = String::from_utf8_lossy(&output.stdout).into_owned();

    // 空のクリップボードの場合は空の配列を返す
    if content.is_empty() {
      return Ok(Vec::new());
    }

    // URIをパースしてファイルパスに変換
    let mut paths = Vec::new();

    for line in content.lines() {
      // URIをトリム
      let line = line.trim();

      // コメント行やからの行をスキップ
      if line.is_empty() || line.starts_with('#') {
        continue;
      }

      // file:// URIをファイルパスに変換
      if line.starts_with("file://") {
        let path = line.trim_start_matches("file://");
        paths.push(path.to_string());
      }
    }

    // 有効なパスが見つからなくても空の配列を返す
    Ok(paths)
  } else {
    let error = String::from_utf8_lossy(&output.stderr).into_owned();
    Err(Error::new(
      ErrorKind::Other,
      format!("Failed to read clipboard for file paths: {}", error),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::temp_dir;
  use std::fs::File;
  use std::path::Path;

  // URI生成の基本的なテスト
  #[test]
  fn test_uri_generation() {
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_uri.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    let path_str = test_file_path.to_string_lossy().to_string();
    let canonical_path = test_file_path.canonicalize().unwrap();
    let expected_uri = format!("file://{}", canonical_path.to_string_lossy());

    let mut uris = Vec::new();
    let path = Path::new(&path_str);
    if let Ok(abs_path) = path.canonicalize() {
      let uri = format!("file://{}", abs_path.to_string_lossy());
      uris.push(uri);
    }

    assert_eq!(uris.len(), 1);
    assert_eq!(uris[0], expected_uri);

    let _ = std::fs::remove_file(test_file_path);
  }

  // 不正なパスを扱えるかのテスト
  #[test]
  fn test_invalid_paths() {
    let invalid_paths = vec![
      "/path/does/not/exist/linux.txt".to_string(),
      "invalid-path-linux.txt".to_string(),
    ];

    // copy_files_to_clipboard を呼び出すが、エラーが発生することを期待
    let result = write_clipboard_file_paths(&invalid_paths);
    assert!(result.is_err());

    // エラーの種類とメッセージを検証
    if let Err(err) = result {
      assert_eq!(err.kind(), ErrorKind::InvalidInput);
      assert!(err
        .to_string()
        .contains("Some paths could not be processed"));
    }
  }

  // 実際のクリップボード操作テスト（CIではスキップ推奨）
  #[test]
  fn test_copy_to_clipboard() {
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_clipboard.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = write_clipboard_file_paths(&[path_str]);

    // xclip がない環境や X11 がない環境では失敗することがある
    // その場合はテストをパスさせるか、環境に応じた処理が必要
    if let Err(e) = &result {
      if e.to_string().contains("Failed to execute xclip command")
        || e.to_string().contains("X11 server connection timed out")
        || e.to_string().contains("No text property")
      // Wayland で発生しうるエラー
      {
        println!("⚠️ クリップボードテストをスキップ: 環境の問題 ({})", e);
        return;
      }
    }

    assert!(result.is_ok(), "Copy operation failed: {:?}", result);

    let _ = std::fs::remove_file(test_file_path);
  }
}
