#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

// OS別の実装モジュール
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

/// Hello World関数 - 動作確認用
#[napi]
pub fn hello_world() -> String {
  "Hello from Rust!".to_string()
}

/// 複数ファイルをクリップボードにコピーする
#[napi]
pub fn copy_files(paths: Vec<String>) -> napi::Result<()> {
  if paths.is_empty() {
    return Err(napi::Error::from_reason("No file paths provided"));
  }

  #[cfg(target_os = "windows")]
  {
    windows::copy_files_to_clipboard(&paths)
      .map_err(|e| napi::Error::from_reason(format!("Windows clipboard error: {}", e)))?;
  }

  #[cfg(target_os = "macos")]
  {
    macos::copy_files_to_clipboard(&paths)
      .map_err(|e| napi::Error::from_reason(format!("macOS clipboard error: {}", e)))?;
  }

  #[cfg(target_os = "linux")]
  {
    linux::copy_files_to_clipboard(&paths)
      .map_err(|e| napi::Error::from_reason(format!("Linux clipboard error: {}", e)))?;
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }

  Ok(())
}
