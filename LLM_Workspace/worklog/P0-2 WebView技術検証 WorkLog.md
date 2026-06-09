# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証

## 現在状態

### 到達点

* WV-08-08 完了

### ドキュメント構成

親:

* WV-08_GTK完全無効化検証.md

子:

* WV-08-01_08-05_GTK基本検証.md
* WV-08-06_08-10_GTK固定Widget検証.md

### 更新対象

現在の更新対象:

* src/platform/linux_webview.rs
* WV-08-06_08-10_GTK固定Widget検証.md

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

## 現在の有力候補

優先度高:

1. GTK Widget追加
2. 継続GTKイベントポンプ
3. WebKitGTK WebView生成
4. WebKitGTK attach
5. WebKitGTK move / resize
6. WebKitGTK + eframe / winit 共存

優先度中:

7. Widget visibility制御
8. Native Surface表示切替

## 次工程

### WV-08-09 GTK Label追加検証

目的:

* GTK Widget追加のみで応答なしが再現するか確認する。

実施内容:

```rust
let label = gtk::Label::new(Some("WV-08-09"));

println!("WV-08-09 label created");

child_fixed.put(&label, 0, 0);

println!("WV-08-09 label attached");
```

判定:

応答なし発生:

* GTK Widget階層が主因候補

応答なし未発生:

* WebKitGTK系へ原因を絞り込む

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. 検証完了済み詳細履歴は技術検証ドキュメントへ移管する
4. WorkLogは現在状態のみ保持する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する
