#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

// 各プラットフォームモジュールで以下の関数を実装する必要があります:
// - copy_files_to_clipboard(&[String]) -> Result<(), Error>
// - read_clipboard_text() -> Result<String, Error>
// - read_clipboard_raw() -> Result<Vec<u8>, Error>
// - read_clipboard_file_paths() -> Result<Vec<String>, Error>
