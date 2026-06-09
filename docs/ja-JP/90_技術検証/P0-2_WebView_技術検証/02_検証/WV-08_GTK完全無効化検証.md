<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260606-000000Z-BCAY
parent_doc_id: 検証目次
lang: ja-JP
canonical_title: WV-08 GTK完全無効化検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-08 GTK完全無効化検証

# WV-08 GTK完全無効化検証

## 目的

WV-07で確認された応答なし現象が、GTK統合層に起因するか、Hyper-V Ubuntu + eframe / winit 側に起因するかを切り分ける。

WV-08では GTK を段階的に再導入し、どの構成要素で応答なしが発生するかを特定する。

## 背景

WV-07では以下を確認した。

* Host Window の位置・サイズ同期は成立した。
* WebView / WebKitGTK / wry を除去しても応答なしが発生した。
* Dummy GTK Widget を除去しても応答なしが発生した。
* Child Widget を除去しても応答なしが発生した。
* 実行中の `move_()` / `resize()` を停止しても応答なしが発生した。
* GTKイベントポンプを停止すると GTK Host Window 自体が表示されなかった。

このため、WV-08では GTK を完全無効化した状態から段階的に機能を追加し、原因箇所を特定する。

## 前提条件

* Hyper-V Ubuntu 環境
* GDK_BACKEND=x11
* eframe / egui / egui_dock を利用
* WebKitGTK 未使用状態から検証開始

## 検証サブドキュメント

### GTK基本検証

* WV-08-01 GTK完全無効化
* WV-08-02 GTK Host Window生成
* WV-08-03 Window表示
* WV-08-04 GTK Window保持
* WV-08-05 GTKイベントflush

参照:

* [GTK基本検証](WV-08-01_08-05_GTK基本検証.md)

### GTK固定Widget検証

* WV-08-06 Root Fixed生成
* WV-08-07 Child Fixed生成
* WV-08-08 Child Fixed move/resize
* WV-08-09 GTK Label追加
* WV-08-10 WebKitGTK最小生成

参照:

* [GTK固定Widget検証](WV-08-06_08-10_GTK固定Widget検証.md)

### WebView単発操作検証

* WV-08-11 WebView set_bounds単発検証

参照:

* [WebView set_bounds単発検証](WV-08-11_WebView_set_bounds単発検証.md)

## 現在の知見

WV-08-01 ～ WV-08-11 の結果から、以下は応答なしの主因ではない可能性が高い。

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
* GTK Label生成
* GTK Widget追加
* WebKitGTK生成
* build_gtk()
* GtkFixed attach
* WebView初期化
* WebView set_bounds単発
* GtkFixed配下での size_allocate

### 現在の有力候補

優先度高:

1. WebView set_bounds継続実行
2. 継続GTKイベントポンプ
3. WebKitGTK + eframe / winit 共存
4. WebView visibility制御

優先度中:

5. Native Surface表示切替
6. Dock追従同期処理

## 現在の結論

WV-08-11時点では、以下は正常動作している。

* GTK Window生成
* GTK Fixed階層構築
* GTK Label追加
* WebKitGTK生成
* build_gtk()
* GtkFixed attach
* WebView set_bounds単発

応答なし現象は再現していない。

GTK基盤および WebKitGTK初期化経路は正常動作している可能性が高い。

問題は継続的な再配置処理、表示制御、継続GTKイベントポンプ、または eframe / winit との共存処理に存在する可能性が高い。

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

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-08 GTK完全無効化検証
