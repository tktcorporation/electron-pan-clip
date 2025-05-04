#![cfg(target_os = "macos")]

// 内部モジュールの定義
mod wrapper;
mod api;
mod tests;

// 公開API
pub use api::{
    copy_files_to_clipboard,
    read_clipboard_text,
    read_clipboard_file_paths,
    read_clipboard_raw
};

// テスト用の公開API
#[cfg(test)]
pub use api::{ClipboardOperations, MockClipboard}; 