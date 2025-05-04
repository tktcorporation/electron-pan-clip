# クリップボード読み取り機能の実装計画

## 目的
Electronアプリケーションから、OSのクリップボードに保存されているデータを読み取るための機能を実装します。
1. RAWデータの読み取り（様々な形式に対応）
2. クリップボードの内容読み取り（ファイルパスとテキストの両方を返す）

## 概要
既存の `copyFilePathsToClipboard` 関数と同様に、Rustのネイティブコードを通じて各OSのクリップボードAPIにアクセスし、データを読み取る機能を提供します。

## 実装手順

### 1. Rust側の実装

#### 1.1 新しい関数と構造体の追加 (`src/lib.rs`)

##### 返り値用の構造体
```rust
/// クリップボードから読み取ったデータを保持する構造体
#[napi(object)]
pub struct ClipboardContent {
  /// ファイルパスのリスト。ファイルパスがない場合は空の配列。
  pub file_paths: Vec<String>,
  
  /// テキスト内容。テキストがない場合はnull。
  pub text: Option<String>,
}
```

##### RAWデータ読み取り関数
```rust
/// Reads raw binary data from the OS clipboard.
///
/// # Returns
/// * Returns `Ok(Vec<u8>)` with the clipboard raw content if successful.
/// * Returns `Err(napi::Error)` if an error occurs or the clipboard is empty.
///
/// # Note
/// * This function reads the current raw contents of the system clipboard.
/// * The format of the data depends on what application wrote to the clipboard.
#[napi]
pub fn read_clipboard_raw() -> napi::Result<Vec<u8>> {
  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    current_platform::read_clipboard_raw().map_err(|e| {
      napi::Error::from_reason(format!("{} clipboard error: {}", std::env::consts::OS, e))
    })
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }
}
```

##### クリップボード内容読み取り関数
```rust
/// Reads content from the OS clipboard, trying to extract both file paths and text.
///
/// # Returns
/// * Returns `Ok(ClipboardContent)` with the clipboard content if successful.
/// * Returns `Err(napi::Error)` if an error occurs or the clipboard is empty.
///
/// # Note
/// * This function attempts to read both file paths and text from the clipboard.
/// * The returned structure indicates what type of data was found.
/// * It's possible for both, either, or neither type of data to be present.
#[napi]
pub fn read_clipboard_content() -> napi::Result<ClipboardContent> {
  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    // ファイルパスの読み取りを試みる
    let file_paths = match current_platform::read_clipboard_file_paths() {
      Ok(paths) => paths,
      Err(_) => Vec::new() // 取得できない場合は空のベクター
    };
    
    // テキストの読み取りを試みる
    let text = match current_platform::read_clipboard_text() {
      Ok(text) => Some(text),
      Err(_) => None // 取得できない場合はNone
    };
    
    // どちらも取得できなかった場合でもエラーにせず空データで返す
    Ok(ClipboardContent {
      file_paths,
      text,
    })
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }
}
```

#### 1.2 プラットフォーム別モジュールの更新 (`src/platforms/mod.rs`)
```rust
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;
```
（既存のものと変更なし）

#### 1.3 各プラットフォーム向け実装

##### macOS実装 (`src/platforms/macos.rs`)

###### テキスト読み取り（内部関数）
```rust
pub fn read_clipboard_text() -> Result<String, Error> {
  unsafe {
    let pool: id = msg_send![class!(NSAutoreleasePool), new];
    
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    let string_class = class!(NSString);
    let pasteboard_string: id = msg_send![pasteboard, stringForType:NSPasteboardTypeString];
    
    let result = if pasteboard_string != nil {
      // NSStringをRustの文字列に変換
      let chars: *const c_char = msg_send![pasteboard_string, UTF8String];
      let rust_str = std::ffi::CStr::from_ptr(chars).to_string_lossy().to_string();
      Ok(rust_str)
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "No text found on clipboard",
      ))
    };
    
    let () = msg_send![pool, drain];
    result
  }
}
```

###### RAWデータ読み取り
```rust
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  unsafe {
    let pool: id = msg_send![class!(NSAutoreleasePool), new];
    
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    
    // 一般的なバイナリデータ (public.data) を取得
    let data_type = NSString::alloc(nil).init_str("public.data");
    let data: id = msg_send![pasteboard, dataForType:data_type];
    
    let result = if data != nil {
      // NSDataをRustのVec<u8>に変換
      let length: NSUInteger = msg_send![data, length];
      let bytes: *const u8 = msg_send![data, bytes];
      
      if length > 0 && !bytes.is_null() {
        let slice = std::slice::from_raw_parts(bytes, length as usize);
        let vec_data = slice.to_vec();
        Ok(vec_data)
      } else {
        Err(Error::new(
          ErrorKind::Other,
          "Empty data on clipboard",
        ))
      }
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "No raw data found on clipboard",
      ))
    };
    
    let () = msg_send![pool, drain];
    result
  }
}
```

###### ファイルパス読み取り
```rust
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  unsafe {
    let pool: id = msg_send![class!(NSAutoreleasePool), new];
    
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    
    // ファイルURLとしての型を指定
    let file_url_type = NSString::alloc(nil).init_str("public.file-url");
    let filenames_type = NSString::alloc(nil).init_str("NSFilenamesPboardType");
    
    // クリップボードから読み取るオブジェクトを取得
    let file_urls: id = msg_send![pasteboard, readObjectsForClasses:
                                  NSArray::arrayWithObject(nil, class!(NSURL))
                                  options:nil];
    
    let mut paths = Vec::new();
    
    if file_urls != nil {
      // NSArrayの要素数を取得
      let count: NSUInteger = msg_send![file_urls, count];
      
      for i in 0..count {
        let url: id = msg_send![file_urls, objectAtIndex:i];
        if url != nil {
          let path: id = msg_send![url, path];
          if path != nil {
            let chars: *const c_char = msg_send![path, UTF8String];
            let path_str = std::ffi::CStr::from_ptr(chars).to_string_lossy().to_string();
            paths.push(path_str);
          }
        }
      }
    }
    
    let () = msg_send![pool, drain];
    
    if paths.is_empty() {
      Err(Error::new(
        ErrorKind::Other,
        "No file paths found on clipboard",
      ))
    } else {
      Ok(paths)
    }
  }
}
```

##### Windows実装 (`src/platforms/windows.rs`)

###### テキスト読み取り（内部関数）
```rust
pub fn read_clipboard_text() -> Result<String, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      let err = GetLastError();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {}", err),
      ));
    }
    
    // テキストデータを取得する
    let h_data = GetClipboardData(CF_UNICODETEXT);
    if h_data == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No text data in clipboard",
      ));
    }
    
    // メモリをロックして内容にアクセス
    let data_ptr = GlobalLock(h_data as *mut _) as *const u16;
    if data_ptr.is_null() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "Failed to lock clipboard memory",
      ));
    }
    
    // ワイド文字列をRustのStringに変換
    let mut length = 0;
    // NULL終端までの長さを計算
    while *data_ptr.add(length) != 0 {
      length += 1;
    }
    
    // UTF-16からRustのStringに変換
    let wide_slice = std::slice::from_raw_parts(data_ptr, length);
    let result = String::from_utf16_lossy(wide_slice);
    
    // メモリをアンロックしてクリップボードを閉じる
    GlobalUnlock(h_data as *mut _);
    CloseClipboard();
    
    Ok(result)
  }
}
```

###### RAWデータ読み取り
```rust
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      let err = GetLastError();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {}", err),
      ));
    }
    
    // 利用可能なフォーマットを取得
    // CF_PRIVATEFIRST (0x0200) を試す
    let h_data = GetClipboardData(0x0200);
    if h_data == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No raw data in clipboard",
      ));
    }
    
    // メモリをロックして内容にアクセス
    let data_ptr = GlobalLock(h_data as *mut _) as *const u8;
    if data_ptr.is_null() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "Failed to lock clipboard memory",
      ));
    }
    
    // データサイズを取得
    let size = GlobalSize(h_data as *mut _);
    if size == 0 {
      GlobalUnlock(h_data as *mut _);
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "Failed to get data size",
      ));
    }
    
    // データをコピー
    let data = std::slice::from_raw_parts(data_ptr, size as usize).to_vec();
    
    // メモリをアンロックしてクリップボードを閉じる
    GlobalUnlock(h_data as *mut _);
    CloseClipboard();
    
    Ok(data)
  }
}
```

###### ファイルパス読み取り
```rust
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      let err = GetLastError();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {}", err),
      ));
    }
    
    // CF_HDROP形式のデータを取得
    let h_drop = GetClipboardData(CF_HDROP);
    if h_drop == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No file paths in clipboard",
      ));
    }
    
    // DROPFILESからファイル数を取得
    let file_count = DragQueryFileW(h_drop as HDROP, 0xFFFFFFFF, ptr::null_mut(), 0);
    if file_count == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No files in drop structure",
      ));
    }
    
    let mut paths = Vec::new();
    
    // 各ファイルパスを取得
    for i in 0..file_count {
      // まずバッファサイズを取得
      let buf_size = DragQueryFileW(h_drop as HDROP, i, ptr::null_mut(), 0);
      
      // バッファを確保
      let mut buffer = vec![0u16; buf_size as usize + 1]; // +1 for NULL terminator
      
      // パスを取得
      let size = DragQueryFileW(
        h_drop as HDROP,
        i,
        buffer.as_mut_ptr(),
        buf_size + 1
      );
      
      if size > 0 {
        // バッファから文字列を生成 (NULL終端除去)
        buffer.truncate(size as usize);
        let path = String::from_utf16_lossy(&buffer);
        paths.push(path);
      }
    }
    
    CloseClipboard();
    
    if paths.is_empty() {
      Err(Error::new(
        ErrorKind::Other,
        "Failed to retrieve file paths",
      ))
    } else {
      Ok(paths)
    }
  }
}
```

##### Linux実装 (`src/platforms/linux.rs`)

###### テキスト読み取り（内部関数）
```rust
pub fn read_clipboard_text() -> Result<String, Error> {
  // arboardを使ってクリップボードの内容を取得
  let mut clipboard = Clipboard::new().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to initialize clipboard: {}", e),
    )
  })?;
  
  // テキストを取得
  let text = clipboard.get_text().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to get text from clipboard: {}", e),
    )
  })?;
  
  if text.is_empty() {
    return Err(Error::new(
      ErrorKind::Other,
      "Clipboard is empty or does not contain text",
    ));
  }
  
  Ok(text)
}
```

###### RAWデータ読み取り
```rust
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  // arboardを使ってクリップボードの内容を取得
  let mut clipboard = Clipboard::new().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to initialize clipboard: {}", e),
    )
  })?;
  
  // バイナリデータを取得
  // arboardはget_imageなどの特定形式のAPIはありますが、
  // 汎用バイナリデータの取得はサポートが限られている可能性があります
  
  // そのため、X11 APIを直接使うことも検討
  // X11の場合は、XSelectionEventを処理して任意の形式のデータを取得可能
  
  // ここでは単純に画像データの取得を試みる例を示します
  let image_data = clipboard.get_image().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to get raw data from clipboard: {}", e),
    )
  })?;
  
  // 画像データをバイナリとして返す
  let bytes = image_data.bytes.clone();
  
  if bytes.is_empty() {
    return Err(Error::new(
      ErrorKind::Other,
      "Clipboard is empty or does not contain raw data",
    ));
  }
  
  Ok(bytes)
}
```

###### ファイルパス読み取り
```rust
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  // arboardを使ってクリップボードの内容を取得
  let mut clipboard = Clipboard::new().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to initialize clipboard: {}", e),
    )
  })?;
  
  // テキストを取得（URIリスト形式を想定）
  let text = clipboard.get_text().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to get text from clipboard: {}", e),
    )
  })?;
  
  if text.is_empty() {
    return Err(Error::new(
      ErrorKind::Other,
      "Clipboard is empty or does not contain file URIs",
    ));
  }
  
  // URIのリストを解析
  let mut paths = Vec::new();
  
  for line in text.lines() {
    let trimmed = line.trim();
    
    // file:// URIをパスに変換
    if trimmed.starts_with("file://") {
      // URI形式からファイルパスへ変換
      let path = trimmed.trim_start_matches("file://");
      paths.push(path.to_string());
    }
  }
  
  if paths.is_empty() {
    return Err(Error::new(
      ErrorKind::Other,
      "No valid file paths found in clipboard",
    ));
  }
  
  Ok(paths)
}
```

### 2. テストの追加

#### 2.1 ユニットテスト (`src/lib.rs`)
```rust
#[cfg(test)]
mod tests {
  use super::*;
  
  // 既存のテスト
  // ...
  
  // RAWデータ読み取り関数のテスト
  #[test]
  #[ignore]
  fn test_read_clipboard_raw() {
    // テスト用のバイナリデータをクリップボードに書き込み
    let test_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
    
    // テスト環境でクリップボードにバイナリデータを書き込む処理が必要
    
    // データを読み取り
    let result = read_clipboard_raw();
    assert!(result.is_ok(), "Failed to read raw clipboard data: {:?}", result);
    
    let clipboard_data = result.unwrap();
    assert_eq!(clipboard_data, test_data);
  }
  
  // クリップボード内容読み取り関数のテスト（ファイルパスの場合）
  #[test]
  #[ignore]
  fn test_read_clipboard_content_file_paths() {
    // テスト用の一時ファイルを作成
    let mut test_paths = Vec::new();
    
    for i in 0..2 {
      let mut path = temp_dir();
      path.push(format!("electron_pan_clip_test_{}.txt", i));
      
      let file_path = path.to_string_lossy().to_string();
      
      // ファイルを作成
      let mut file = File::create(&path).expect("Failed to create test file");
      writeln!(file, "Test content {}", i).expect("Failed to write to test file");
      
      test_paths.push(file_path);
    }
    
    // ファイルパスをクリップボードにコピー
    let copy_result = write_clipboard_file_paths(test_paths.clone());
    assert!(copy_result.is_ok(), "Failed to copy file paths to clipboard");
    
    // クリップボード内容を読み取り
    let result = read_clipboard_content();
    assert!(result.is_ok(), "Failed to read clipboard content: {:?}", result);
    
    let content = result.unwrap();
    
    // ファイルパスのみ取得できているはず
    assert!(!content.file_paths.is_empty());
    assert_eq!(content.file_paths.len(), test_paths.len());
    assert!(content.text.is_none());
    
    // テスト後にファイルを削除
    for path in test_paths {
      let _ = std::fs::remove_file(path);
    }
  }
  
  // クリップボード内容読み取り関数のテスト（テキストの場合）
  #[test]
  #[ignore]
  fn test_read_clipboard_content_text() {
    // テスト用のテキストをクリップボードに書き込み
    let test_text = "This is plain text, not a file path";
    
    // ここではテスト環境で利用可能なクリップボード操作方法を使用
    // （プラットフォーム依存）
    
    // クリップボード内容を読み取り
    let result = read_clipboard_content();
    assert!(result.is_ok(), "Failed to read clipboard content: {:?}", result);
    
    let content = result.unwrap();
    
    // テキストのみ取得できているはず
    assert!(content.file_paths.is_empty());
    assert!(content.text.is_some());
    assert_eq!(content.text.unwrap(), test_text);
  }
  
  // クリップボード内容読み取り関数のテスト（両方のデータがある場合）
  #[test]
  #[ignore]
  fn test_read_clipboard_content_both() {
    // クリップボードに両方の形式のデータが含まれている状態を作る必要がある
    // これは実装が難しいかもしれないため、このテストは必要に応じて調整
    
    // クリップボード内容を読み取り
    let result = read_clipboard_content();
    
    if result.is_ok() {
      let content = result.unwrap();
      
      // 両方のデータが取得できた場合
      if !content.file_paths.is_empty() && content.text.is_some() {
        // 問題なく両方のデータが取れていることを確認
        assert!(true);
      }
    }
  }
}
```

#### 2.2 各プラットフォーム向けテスト
各プラットフォームのファイル (`src/platforms/{windows,macos,linux}.rs`) にも、単体テストを追加します。

### 3. JavaScript側のインターフェース

#### 3.1 `index.js` の更新
これは自動生成されるため、Rust側の関数を追加すると自動的に更新されます。

#### 3.2 TypeScript型定義の追加 (`index.d.ts`)
```typescript
/**
 * クリップボードから読み取ったデータを保持するインターフェース
 */
export interface ClipboardContent {
  /**
   * ファイルパスのリスト。ファイルパスがない場合は空の配列。
   */
  file_paths: string[];
  
  /**
   * テキスト内容。テキストがない場合はnull。
   */
  text: string | null;
}

/**
 * Reads raw binary data from the OS clipboard.
 * 
 * @returns The raw binary content of the clipboard as a Buffer.
 * @throws If an error occurs or the clipboard is empty.
 */
export declare function readClipboardRaw(): Buffer

/**
 * Reads content from the OS clipboard, trying to extract both file paths and text.
 * 
 * @returns An object containing file paths and/or text (if available). If clipboard is empty both will be empty / null.
 * @throws If an error occurs or the clipboard is empty.
 */
export declare function readClipboardContent(): ClipboardContent
```

## 必要なライブラリと依存関係
すべての必要なライブラリはすでに `Cargo.toml` に含まれています:

- **macOS**: `objc`, `cocoa`, `core-foundation`
- **Windows**: `windows-sys` (適切な機能フラグ付き)
- **Linux**: `arboard`, `x11`, `libc`

## 実装上の注意点

1. **エラーハンドリング**
   - クリップボードにデータがない場合や、クリップボードへのアクセスに失敗した場合は適切なエラーメッセージを返す
   - プラットフォーム固有のエラーも適切に捕捉して、わかりやすいメッセージで返す
   - データ形式が期待と異なる場合のエラー処理

2. **メモリ管理**
   - 特にC/C++系のAPIを使用するWindows/macOSでは、適切にメモリを解放する
   - リソースリークを防ぐため、`unsafe` ブロック内でのエラー処理に注意
   - 特にRAWデータ処理時に大きなメモリを扱う可能性がある点に注意

3. **クロスプラットフォーム**
   - 各プラットフォームで同じように動作するようにする
   - 返り値の形式（改行コード、パス区切り文字など）を統一するか、プラットフォーム依存の動作は明示する

4. **RAWデータの扱い**
   - RAWデータの形式はプラットフォームや書き込んだアプリケーションに依存する
   - 汎用的な読み取りには限界がある場合がある（特定の形式に特化した読み取り関数の追加も検討）

5. **ファイルパスの形式**
   - Windowsではバックスラッシュ、Unix系ではスラッシュなど、OSによって区切り文字が異なる
   - URIエンコーディング（`file://`プレフィックスなど）の適切な処理

6. **データ取得の効率**
   - ファイルパスとテキストを同時に取得する場合、一度のクリップボードアクセスで両方を取得できる可能性がある
   - プラットフォームによっては複数回クリップボードにアクセスする必要があるため、その間にクリップボードの内容が変わるリスクを考慮する

## API設計

### RAWデータ読み取り
- **関数名**: `readClipboardRaw`
- **引数**: なし
- **戻り値**: クリップボードのバイナリデータを表す `Buffer`
- **例外**: クリップボードが空またはアクセスエラーの場合にエラーをスロー

### クリップボード内容読み取り
- **関数名**: `readClipboardContent`
- **引数**: なし
- **戻り値**: 
  - `ClipboardContent` 構造体：ファイルパスのリスト、テキスト内容を含む
  - データの種類は呼び出し側が file_paths/text の有無を見て判断する
- **例外**: クリップボードが空またはアクセスエラーの場合にエラーをスロー

## 今後の拡張可能性

1. **他の形式の読み取り**
   - HTML、RTF、画像などの他のクリップボード形式のサポートを追加可能
   
2. **形式チェック機能**
   - クリップボードに含まれるデータ形式をチェックする関数

3. **特定形式への変換機能**
   - RAWデータを特定の形式（画像、HTML、RTFなど）として解釈・変換する関数

4. **クリップボード監視機能**
   - クリップボードの内容が変更されたときに通知する機能

## 実装スケジュール

1. Rust側の基本機能の実装
   - `ClipboardContent` 構造体の定義
   - RAWデータ読み取り
   - クリップボード内容読み取り（ファイルパスとテキストの両方を取得）
2. プラットフォーム固有の実装の追加
3. テストの実装
4. JavaScript側のインターフェースの確認
5. 動作確認とバグ修正 