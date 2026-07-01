<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000003Z-WV02
lang: ja-JP
canonical_title: WV-02 egui共存
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-02 egui共存

# WV-02 egui共存

## 目的

egui UI と WebView の共存可能性を確認する。

本検証では、WV-01 にて成立した WebView 表示を実際の Workflow IDE Framework に近い構成へ適用し、egui_dock と Child Window Overlay 方式の共存が成立することを確認する。

まずは Windows を対象として成立性を確認し、その結果を基に Linux および macOS で同一検証を実施する。

## 前提条件

### WV-01

WV-01 により以下を確認済みである。

- WebView生成成功
- example.com表示成功
- build_as_child(frame) 成功
- eframe と wry の共存成功

### 確認対象

- Dock移動
- Dockリサイズ
- Dockタブ切替
- Floating Panel
- WebView再配置
- 入力競合

## PoC

### 検証内容

#### PoC-2e Dock配置確認

確認項目

- Dock移動
- Dockリサイズ
- Dockタブ切替
- Floating Panel
- WebView再配置
- 入力競合

#### PoC-2f フローティング禁止方式評価

確認項目

- フローティング無効化可否
- Dock移動への影響
- UXへの影響
- GPU Viewportへの適用可能性

### 実施結果

#### PoC-2e Dock配置確認

確認結果

- Dock移動成功
- Dockリサイズ成功
- Dockタブ切替成功
- WebView Child Window追従成功
- WebViewリサイズ追従成功
- WebViewタブ非表示時 Hide 成功
- 他タブの WebView Panel へのドラッグ成功
- Dock操作と入力競合は解消
- WebView Panel へのタブドラッグ時のみ WebView Hide 成功
- タブドラッグ終了後の表示復元成功
- WebView非アクティブ時 Hide 成功
- WebViewアクティブ時 Show 成功
- 初回表示時の左上フラッシュ解消
- Child Window 初期配置後に WebView生成する方式成立

確認された課題

- Floating Panel 表示時にネイティブサーフェスとの Z オーダー競合が発生する
- 本課題は WebView 固有ではなく GPU Viewport 等のネイティブサーフェス全般に共通する

判定

条件付き成功

#### PoC-2f フローティング禁止方式評価

確認結果

- フローティング禁止可能
- Dock移動への影響なし
- UXへの重大な影響なし
- GPU Viewportへの適用可能

確認された課題

なし

判定

成功

### 評価

egui_dock と WebView の共存は成立した。

また Dock移動、Dockリサイズ、Dockタブ切替、WebView追従についても問題なく動作することを確認した。

一方で Child Window Overlay 方式では、Floating Panel とネイティブサーフェス間で Z オーダー競合が発生することが判明した。

PoC-2f により、egui_dock の `allowed_in_windows()` を利用して Floating Panel を禁止できることを確認したため、本課題は回避可能と判断する。

また、WebView Panel へのタブドラッグ時にのみ Child Window を退避し、
配置完了後はアクティブタブ状態に応じて表示制御できることを確認した。

さらに Child Window を DockRect へ配置後に WebView を生成する方式により、
初回表示時のネイティブサーフェスのフラッシュを解消できることを確認した。

### 後続検証

- WV-03 Linux egui共存
- WV-04 macOS egui共存

## WV評価

### 判定

成功

### 根拠

確認済み事項

- egui_dock と WebView の共存成立
- Dock移動成功
- Dockリサイズ成功
- Dockタブ切替成功
- Child Window追従成功
- WebViewリサイズ追従成功
- Dock操作との入力競合解消
- Floating禁止方式成立
- WebView Panel へのタブドラッグ時のみ Hide 成功
- アクティブタブ判定成功
- 非アクティブタブ時 Hide 成功
- 初回表示時のフラッシュ解消
- Child Window 初期配置方式成立

### 制約

- Child Window Overlay 方式を利用するネイティブサーフェスは Floating Panel と共存できない。
- ネイティブサーフェスはアクティブタブ時のみ表示する
- タブドラッグ中は一時退避が必要

対象例

- WebView
- GPU Viewport
- Video Surface
- DirectX Surface
- Vulkan Surface

### 設計判断

Workflow IDE Framework では以下を設計方針候補とする。

- 通常の egui Panel は Floating Panel を許可する
- ネイティブサーフェスを含む Panel は Dock 内のみ許可する
- ネイティブサーフェスはアクティブタブ時のみ表示する
- タブドラッグ中はネイティブサーフェスを一時退避する
- Child Window は DockRect 配置後に生成する

期待効果

- Z オーダー競合回避
- Child Window Overlay 方式継続利用
- GPU Viewportへの横展開
- UI操作性維持

## 次工程

### WV-03 Linux egui共存

確認事項

- cargo build
- アプリ起動
- Dock矩形取得
- Dock移動
- Dockリサイズ
- WebView生成
- URL表示
- タブ切替
- Child Window追従
- Floating禁止方式
- WebKitGTK依存確認

### WV-04 macOS egui共存

確認事項

- cargo build
- アプリ起動
- Dock矩形取得
- Dock移動
- Dockリサイズ
- WebView生成
- URL表示
- タブ切替
- Child Window追従
- Floating禁止方式

## 備考

本検証により、WebView Support Panel の成立性は確認できた。

残課題は Child Window Overlay 方式における Z オーダー制御であるが、Floating Panel を禁止することで実運用上の回避が可能である。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-02 egui共存
