# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証

## 現在状態

### 到達点

* WV-08-11 完了

### ドキュメント構成

親:

* WV-08_GTK完全無効化検証.md

子:

* WV-08-01_08-05_GTK基本検証.md
* WV-08-06_08-10_GTK固定Widget検証.md
* WV-08-11_WebView_set_bounds単発検証.md

### 更新対象

現在の更新対象:

* src/platform/linux_webview.rs
* WV-08_GTK完全無効化検証.md

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

## 現在の有力候補

優先度高:

1. WebView set_bounds継続実行
2. 継続GTKイベントポンプ
3. WebKitGTK + eframe / winit 共存
4. WebView visibility制御

優先度中:

5. Native Surface表示切替
6. Dock追従同期処理

## 次工程

### WV-08-12 WebView set_bounds継続追従検証

目的:

* Dock矩形変化に追従して WebView::set_bounds() を継続実行した場合に応答なしが再現するか確認する。

実施内容:

* build_gtk()
* Dock矩形取得
* set_bounds継続実行

実施しない内容:

* set_visible()
* Native Surface表示切替

判定:

応答なし発生:

* 継続的な set_bounds() が主因候補

応答なし未発生:

* visibility制御またはGTKイベントポンプへ調査を進める

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. 検証完了済み詳細履歴は技術検証ドキュメントへ移管する
4. WorkLogは現在状態のみ保持する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する
