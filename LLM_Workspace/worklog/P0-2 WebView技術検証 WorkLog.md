# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証

## 現在状態

### 到達点

* WV-09-06-04 完了
* WV-09-06-03 完了
* WV-09-06-02 完了
* WV-09-06-01 完了
* WV-09-05 完了
* WV-09-04 完了
* WV-09-03 完了
* WV-09-02 完了
* WV-09-01 完了
* WV-08 系検証完了

### ドキュメント構成

親:

* WV-09_Linux応答なし原因特定.md

子:

* WV-09-01_WV07_WV08差分分析.md
* WV-09-02_X11親子Window制御検証.md
* WV-09-03_GtkFixed階層検証.md
* WV-09-04_WebKitGTK_eframe_winit共存検証.md
* WV-09-05_Native_Child_Window再導入検証.md
* WV-09-06_実運用同期処理切り分け検証.md
* WV-09-06-04_WebView_visibility継続実行停止検証.md

### 更新対象

現在の更新対象:

* src/platform/linux_webview.rs
* WV-09-06_実運用同期処理切り分け検証.md
* WV-09-06-04_WebView_visibility継続実行停止検証.md
* 検証目次.md

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
* WebView set_bounds単発
* WebView set_bounds継続実行
* WebView set_visible(true)
* sync_child_window() 経由の WebView set_visible(true)
* WebView set_visible(false)
* sync_child_window() 経由の WebView set_visible(false)
* WebView set_visible継続実行
* flush_gtk_events_throttled()
* gtk::main_iteration_do(false) 継続実行
* Visibility同期フェーズの GTKイベントポンプ
* Native Surface表示切替
* GTK Host Window単体
* X11 Host Window同期
* GtkFixed階層
* WebKitGTK + eframe / winit 共存
* Native Child Window再導入

## 現在の有力候補

優先度高:

1. Native Surface位置同期
2. GTK Window show / hide
3. GTKイベントポンプとの組み合わせ
4. 現在の develop では再現条件を失っている可能性

## 次工程

### WV-09-06-05 Native Surface位置同期単独有効検証

目的:

* Native Surface位置同期を単独有効化し、Linux応答なしが再現するか確認する。

判定:

応答なし発生:

* Native Surface位置同期を主因候補として扱う

応答なし未発生:

* Native Surface位置同期単独を主因候補から除外し、GTK Window show / hide または複合要因の調査へ進む

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. 検証完了済み詳細履歴は技術検証ドキュメントへ移管する
4. WorkLogは現在状態のみ保持する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する
