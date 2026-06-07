# P0-2 WebView技術検証 WorkLog

## 対象

* リポジトリ: workflow-ide-framework
* テーマ: P0-2 WebView技術検証
* 対象ファイル:

  * src/platform/linux_webview.rs

## 現在の到達点

### WV-07

目的:

* GTK Host Window + Child Surface + WebView の統合検証

結果:

* Host Window生成成功
* Host Window追従成功
* Dock移動追従成功
* Dockサイズ変更追従成功
* WebView生成成功
* Host Window上でマウス移動時に応答なし発生

結論:

* GTK統合層周辺に問題が存在
* まず GTK構成要素を段階的に除去して切り分けを実施

---

### WV-08

目的:

* 応答なし発生箇所の切り分け

---

#### WV-08-01

構成:

* GTK完全無効
* gtk::init()なし
* GTK Windowなし
* WebViewなし
* GTKイベントポンプなし

結果:

* PoC-2e動作
* Dock移動可能
* Dockサイズ変更可能
* 応答なし発生せず

結論:

* GTK統合層に原因が存在

---

#### WV-08-02

構成:

* gtk::init()のみ実行

結果:

* gtk::init()成功
* PoC-2e動作
* Dock移動可能
* Dockサイズ変更可能
* 応答なし発生せず

結論:

* gtk::init()は原因ではない

---

#### WV-08-03

構成:

* gtk::init()
* gtk::Window::new()

のみ実行

以下は未実施

* show_all()
* move_
* resize
* GTKイベントポンプ
* Child Widget
* WebView

結果:

* gtk::Window created
* PoC-2e動作
* Dock移動可能
* Dockサイズ変更可能
* 応答なし発生せず

結論:

* gtk::Window生成は原因ではない

---

## 現在の除外済み要因

以下は原因ではないことを確認済み

* PoC-2e
* egui
* egui_dock
* eframe
* winit
* gtk::init()
* gtk::Window::new()

---

## 次回実施

### WV-08-04

目的:

* show_all()単独追加による切り分け

構成:

* gtk::init()
* gtk::Window::new()
* window.show_all()

以下は未実施

* move_
* resize
* GTKイベントポンプ
* Child Widget
* WebView

判定:

* 応答なし発生

  * show_all() / realize系が原因候補

* 応答なし未発生

  * GTKイベントポンプ
  * move_
  * resize
  * Child Widget
  * WebView

のいずれかが原因候補

---

## 最新ソース

* linux_webview(37).rs

## 運用ルール

1. 最新ソースを正とする

2. AIは次の検証内容を決定する

3. AIが全文差し替え版を作成できない場合は最小差分を提示する

4. ユーザーが編集して検証する

5. ユーザーは結果のみ返却する

6. 最新ソースが変更された場合のみ再添付する
