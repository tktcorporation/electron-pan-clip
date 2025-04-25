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

# Xvfbがすでに実行されているか確認
if ! pgrep -x "Xvfb" > /dev/null; then
  echo "仮想X11サーバー（Xvfb）を起動中..."
  Xvfb $XVFB_DISPLAY -screen 0 1280x1024x24 &
  XVFB_PID=$!
  
  # 少し待機して確実に起動するようにする
  sleep 1
  
  # プロセスが起動しているか確認
  if ! ps -p $XVFB_PID > /dev/null; then
    echo "エラー: Xvfbの起動に失敗しました"
    exit 1
  fi
  
  # このスクリプトが終了したらXvfbも終了させる
  trap "kill $XVFB_PID" EXIT
else
  echo "Xvfbはすでに実行中です"
fi

# DISPLAYをエクスポート
export DISPLAY=$XVFB_DISPLAY

# 渡されたコマンドを実行
echo "コマンドを実行中: $@"
"$@"

# 終了コードを保存
EXIT_CODE=$?

echo "実行が完了しました（終了コード: $EXIT_CODE）"
exit $EXIT_CODE
