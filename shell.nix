{ pkgs ? import <nixpkgs> {} }:

let
  # OS判定用の変数
  isDarwin = pkgs.stdenv.isDarwin;
  isLinux = pkgs.stdenv.isLinux;
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # 共通の基本ツール (全OS)
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    cargo-binstall
    
    # Node.js環境
    nodejs_22
    yarn-berry
    nodePackages.typescript
    nodePackages.yarn
    nodePackages."@antfu/ni"
    
    # Zig (napi-rs用)
    zig
    
    # GitHub関連
    gh
    act
    
    # タスクランナー
    just
    
    # 基本開発ツール
    pkg-config
    openssl.dev
    git
  ] 
  # macOS固有のツール
  ++ pkgs.lib.optionals isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.CoreFoundation
    darwin.apple_sdk.frameworks.CoreServices
    # Cargo.tomlで指定されているmacOS依存
    darwin.apple_sdk.frameworks.AppKit # cocoa用
  ]
  # Linux固有のツール
  ++ pkgs.lib.optionals isLinux [
    xorg.libX11
    xorg.libXi
    xorg.libXtst
    xorg.libXext
  ];

  # 環境変数（共通）
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  
  # OS固有の環境変数
  shellHook = ''
    ${if isLinux then ''
      export DISPLAY=":0"
    '' else ""}
    
    ${if isDarwin then ''
      # macOS固有の設定
      echo "macOS環境を設定しました。"
    '' else ""}
    
    echo "Rust環境が設定されました。"
  '';
} 