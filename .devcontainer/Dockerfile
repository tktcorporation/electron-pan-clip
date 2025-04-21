FROM mcr.microsoft.com/devcontainers/rust:latest

# Linuxで必要なシステムライブラリのインストール（クリップボード操作関連）
RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libgtk-3-dev \
    libssl-dev \
    xclip \
    wl-clipboard \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* 