# WV-11-06 Linux VM 実機検証結果

## 目的

Linux VM 上で、Windows と同じ CEF OSR → egui Texture → Dock の Browser Surface 経路が成立するか確認する。

GTK / WebKitGTK の Host Window 埋め込み方式は使用しない。

## 検証結果

| 動作対象 | CEF OSR | Dock 内表示 | リサイズ追従 |
| --- | --- | --- | --- |
| Linux VM / Wayland | ○ | ○ | ○ |
| Linux 実機 | ？ | ？ | ？ |
| macOS | ？ | ？ | ？ |

Linux VM の GUI セッションでは `DISPLAY=:0`、`WAYLAND_DISPLAY=wayland-0` が設定された状態で検証した。

### CEF OSR

`cef_buffer_probe` が正常にビルド・起動し、800x600 / 1,920,000 bytes の Paint buffer を取得した。

結果: `○`

### Dock 内表示

`cef_dock_probe` が正常に起動し、CEF OSR Paint を受信して 800x600 の egui Texture を生成した。画面上でも Browser Surface の Dock 内表示を確認した。

結果: `○`

### リサイズ追従

`cef_dock_resize_probe` でアプリケーション Window を繰り返しリサイズし、CEF OSR の resize request に追従して Paint size が更新され続けることを確認した。画面上でも Browser Surface の追従動作を確認した。

結果: `○`

## 観測事項

VM の Wayland / 仮想 GPU 環境では、以下の警告・エラーが CEF / Mesa から出力された。

- DRM render node を取得できない旨のログ
- EGL / Mesa / ZINK の初期化警告
- SharedImage 関連エラー
- resize 中の GPU process exit (`exit_code=5`)

ただし、その後も CEF OSR Paint は継続し、Dock 表示およびリサイズ追従は成立した。このため、今回の Linux VM 成立性判定は `○` とする。

これらの GPU 関連ログについては Linux 実機または異なる VM GPU 構成で再現性を確認する余地があるが、現時点では Browser Surface の成立を妨げる事象として扱わない。

## ビルド用 target directory

当初 `/tmp/workflow-ide-p0-2-target` を使用したところ、VM の `/tmp` が約 1.7 GiB の tmpfs であり、CEF を含む依存関係の初回ビルド中に `Disk quota exceeded (os error 122)` が発生した。

これは Browser Surface の技術的不成立ではなく検証環境の容量不足である。

`CARGO_TARGET_DIR=$HOME/cargo-target/workflow-ide-p0-2` に変更後、ビルドおよび3検証は完走した。この結果を受け、Linux 検証ランナーの既定 target directory も `$HOME/cargo-target/workflow-ide-p0-2` へ変更する。

## 判定

Linux VM / Wayland では CEF OSR → egui Texture → Dock の Browser Surface 基本経路が成立する。

Linux 実機および macOS は環境未検証のため `？` とし、非対応とは判定しない。

WV-11-06 のうち、現在利用可能な Linux VM 環境での成立性確認は完了とする。
