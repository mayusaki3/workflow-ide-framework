# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証

## 現在状態

### 到達点

* WV-08-17 完了
* WV-08 系検証完了

### ドキュメント構成

親:

* WV-08_GTK完全無効化検証.md

子:

* WV-08-01_08-05_GTK基本検証.md
* WV-08-06_08-10_GTK固定Widget検証.md
* WV-08-11_WebView_set_bounds単発検証.md
* WV-08-12_WebView_set_bounds継続追従検証.md
* WV-08-13_WebView_visibility追従検証.md
* WV-08-14_sync_child_window_visibility検証.md
* WV-08-15_WebView_set_visible_false強制検証.md
* WV-08-16_GTKイベントポンプ再導入検証.md
* WV-08-17_Native_Surface表示切替単独検証.md

### 更新対象

現在の更新対象:

* src/platform/linux_webview.rs
* WV-08_GTK完全無効化検証.md
* WV-08-12_WebView_set_bounds継続追従検証.md
* WV-08-13_WebView_visibility追従検証.md
* WV-08-14_sync_child_window_visibility検証.md
* WV-08-15_WebView_set_visible_false強制検証.md
* WV-08-16_GTKイベントポンプ再導入検証.md
* WV-08-17_Native_Surface表示切替単独検証.md

親ドキュメントは知見管理のみ行う。

## 除外済み要因

* gtk::init()
* gtk::Window::new()
* window.show_all()
* GTK Window保持
* 単発GTKイベントflush
* gtk::main_iteration_do(false)
* gtk::Fixed::new()
* Root Fixed attach
* Child Fixed attach
* Fixed階層構築
* Child Fixed move
* Child Fixed resize
* GTKレイアウト更新
* GTK Label生成
* GTK Widget追加
* WebKitGTK生成
* build_gtk()
* GtkFixed attach
* WebView初期化
* WebView set_bounds単発
* GtkFixed配下での size_allocate
* WebView set_bounds継続実行
* WebView set_visible(true)
* sync_child_window() 経由の WebView set_visible(true)
* WebView set_visible(false)
* sync_child_window() 経由の WebView set_visible(false)
* flush_gtk_events_throttled()
* gtk::main_iteration_do(false) 継続実行
* Native Surface表示切替

## 現在の有力候補

優先度高:

1. WebKitGTK + eframe / winit 共存
2. X11親子ウィンドウ制御
3. 実運用側の同期処理

優先度中:

4. Dock追従同期処理

## 次工程

### WV-09 系検証へ移行

目的:

* WV-07で応答なしが発生した構成との差分を再確認し、WV-08で除外できなかった要素を切り分ける。

調査候補:

* WebKitGTK + eframe / winit 共存
* X11親子ウィンドウ制御
* 実運用側の同期処理
* Dock追従同期処理

判定:

応答なし発生:

* 追加した構成要素を主因候補として扱う

応答なし未発生:

* 次の未評価要素へ調査を進める

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. 検証完了済み詳細履歴は技術検証ドキュメントへ移管する
4. WorkLogは現在状態のみ保持する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する
