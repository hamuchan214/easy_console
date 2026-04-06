# easy_console

Rust製のシリアルコンソールです。macOSで動きます。たぶんWindowsとLinuxでも動く。

普通のシリアルコンソール（minicomとかscreen）は設定がめんどくさかったり、見た目がいまいちだったりするので自分で作りました。

## インストール

```bash
git clone https://github.com/hamuchan214/easy_console
cd easy_console
make install
```

Rustが入ってれば`make install`で`easy_console`コマンドが使えるようになります。

## 使い方

```bash
# ポート一覧を見る
easy_console --list-ports

# 接続
easy_console -p /dev/cu.usbserial-0001 -b 115200
```

起動するとこんな感じのTUI画面になります。F2でポート選択、F3で設定変更ができます。

## キー操作

よく使うやつだけ書いておきます。

| キー | 動作 |
|------|------|
| F2 | ポート選択 |
| F3 | ボーレートとかの設定 |
| F4 | ASCII / HEX / SPLIT 表示切り替え |
| F5 | ログファイル保存のON/OFF |
| F6 | マクロ |
| Ctrl+L | 画面クリア |
| Ctrl+E | 改行コード切り替え（CRLF→LF→CR→None） |
| Ctrl+I | 統計パネル（DTR/RTSとかの信号も見れるはず） |
| / | ログ内検索 |
| Ctrl+Q | 終了 |

## 設定ファイル

`~/.config/easy_console/profiles.toml` にプロファイルを書いておくと便利です。

```toml
[profiles.default]
port = "/dev/cu.usbserial-0001"
baud_rate = 115200
tx_newline = "crlf"
timestamp = true
```

起動時に `--profile default` で読み込めます。

## 開発者向け機能

- HEXダンプ表示（F4で切り替え）
- 改行コードを送受信で個別に設定できる
- マクロ（複数コマンドをウェイト付きで連続送信）
- RS-232制御線のリアルタイム表示
- 正規表現でログを検索・フィルタ

## ビルド

```bash
cargo build --release
```

## License

MIT
