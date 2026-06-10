# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証

## 現在状態

### 到達点

* WV-08-14 完了

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

### 更新対象

現在の更新対象:

* src/platform/linux_webview.rs
* WV-08_GTK完全無効化検証.md
* WV-08-12_WebView_set_bounds継続追従検証.md
* WV-08-13_WebView_visibility追従検証.md
* WV-08-14_sync_child_window_visibility検証.md

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

## 現在の有力候補

優先度高:

1. WebView set_visible(false)
2. 継続GTKイベントポンプ
3. WebKitGTK + eframe / winit 共存
4. Native Surface表示切替

優先度中:

5. Dock追従同期処理

## 次工程

### WV-08-15 WebView set_visible(false) 強制検証

目的:

* Native Surface非表示経路を強制実行し、WebView::set_visible(false) が応答なし要因か確認する。

実施内容:

* build_gtk()
* Dock追従
* set_bounds継続実行
* set_visible(false) 強制実行

判定:

応答なし発生:

* set_visible(false) が主因候補

応答なし未発生:

* visibility制御は主因から除外し、継続GTKイベントポンプまたは eframe / winit 共存へ調査を進める。

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. 検証完了済み詳細履歴は技術検証ドキュメントへ移管する
4. WorkLogは現在状態のみ保持する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する
