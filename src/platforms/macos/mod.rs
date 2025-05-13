// 内部モジュールの定義
mod api;
mod tests;
mod wrapper;

// 公開API
pub use api::{
  read_clipboard_file_paths, read_clipboard_raw, read_clipboard_text, write_clipboard_file_paths,
};

// テスト用の公開API
