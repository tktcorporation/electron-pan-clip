#!/bin/bash

# Xvfbを使用してテストを実行するためのスクリプト
# このスクリプトは仮想X11サーバーを起動し、その環境でコマンドを実行します

# ディスプレイ番号を設定
XVFB_DISPLAY=":99"

# X11-unixディレクトリが存在するか確認し、存在しない場合は作成
if [ ! -d /tmp/.X11-unix ]; then
  echo "/tmp/.X11-unix ディレクトリを作成中..."
  sudo mkdir -p /tmp/.X11-unix
  sudo chmod 1777 /tmp/.X11-unix
fi

# 既存のXvfbプロセスをクリーンアップ
pkill -f "Xvfb $XVFB_DISPLAY" > /dev/null 2>&1

# Xvfbを確実に起動する
echo "仮想X11サーバー（Xvfb）を起動中..."
Xvfb $XVFB_DISPLAY -screen 0 1280x1024x24 -ac +extension GLX +render -noreset &
XVFB_PID=$!

# 十分な待機時間を設定（3秒に増加）
echo "Xvfbの起動を待機中..."
sleep 3

# プロセスが起動しているか確認
if ! ps -p $XVFB_PID > /dev/null; then
  echo "エラー: Xvfbの起動に失敗しました"
  exit 1
fi

# このスクリプトが終了したらXvfbも終了させる
trap "echo '仮想X11サーバーを終了中...'; kill $XVFB_PID" EXIT

# X11関連の環境変数を設定
export DISPLAY=$XVFB_DISPLAY
export XAUTHORITY=/tmp/.Xauthority
export XDG_RUNTIME_DIR=/tmp/xdg-runtime-dir
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

# クリップボード関連のデーモンを起動（利用可能な場合）
if command -v xclip > /dev/null; then
  echo "xclipが使用可能です"
fi

if command -v dbus-launch > /dev/null; then
  echo "dbus-launchが使用可能です"
  eval $(dbus-launch --sh-syntax)
  echo "DBUSセッションバスが設定されました: $DBUS_SESSION_BUS_ADDRESS"
fi

# X11サーバーが実際に応答するか確認
echo "X11サーバーの応答を確認中..."
if command -v xdpyinfo > /dev/null; then
  xdpyinfo > /dev/null 2>&1
  if [ $? -eq 0 ]; then
    echo "X11サーバーは正常に応答しています"
  else
    echo "警告: X11サーバーが応答していません"
  fi
fi

# 渡されたコマンドを実行
echo "コマンドを実行中: $@"
"$@"

# 終了コードを保存
EXIT_CODE=$?

echo "実行が完了しました（終了コード: $EXIT_CODE）"
exit $EXIT_CODE
