# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証
* 対象ファイル:
  * src/platform/linux_webview.rs

## 現在の到達点

### WV-07

結果:

* GTK Host Window生成成功
* Host Window追従成功
* Dock移動追従成功
* Dockサイズ変更追従成功
* WebView生成成功
* Host Window上でマウス移動時に応答なし発生

結論:

* GTK統合層周辺に問題が存在
* WV-08で段階的切り分けを開始

---

### WV-08 切り分け結果

| 検証 | 内容 | 結果 |
|--------|--------|--------|
| WV-08-01 | GTK完全無効 | 成功 |
| WV-08-02 | gtk::init() | 成功 |
| WV-08-03 | gtk::Window::new() | 成功 |
| WV-08-04 | window.show_all() + Window保持 | 成功 |
| WV-08-05 | 単発GTK flush | 成功 |
| WV-08-06 | Root Fixed生成・attach | 成功 |
| WV-08-07 | Child Fixed生成・attach | 成功 |
| WV-08-08 | Child Fixed move / resize | 成功 |

共通結果:

* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし再現せず

観測事項:

* GTK Windowはデスクトップ左上に表示
* WV-08-08でChild Fixed resizeによりWindowサイズ変化を確認

---

## 現在の除外済み要因

以下は原因ではない可能性が高い。

* PoC-2e
* egui
* egui_dock
* eframe
* winit
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

---

## 現在の有力候補

優先度高:

1. 継続GTKイベントポンプ
2. WebKitGTK WebView生成
3. WebKitGTK attach
4. WebKitGTK move / resize
5. WebKitGTK + egui/winit 共存

優先度中:

6. gtk_widget_show / hide
7. Native Surface表示切替処理

---

## 次回実施候補

### WV-08-09

目的:

* GTK Widget追加のみで問題が再現するか確認

構成:

* GTK Label生成
* child_fixed.put(label)
* WebView未生成

判定:

* 応答なし発生
  * GTK Widget階層が原因候補

* 応答なし未発生
  * WebKitGTK系へ原因を絞り込む

---

## 最新ソース

* linux_webview.rs
* 検証到達点: WV-08-08

## 運用ルール

1. 最新ソースを正とする
2. AIは次の検証内容を決定する
3. AIが全文差し替え版を作成できない場合は最小差分を提示する
4. ユーザーが編集して検証する
5. ユーザーは結果のみ返却する
6. 最新ソースが変更された場合のみ再添付する