<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260531-000004Z-WV28
lang: ja-JP
canonical_title: WV-02 egui共存
document_type: test_result
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [WebView 検証結果](./WebView_検証結果.md) > WV-02 egui共存

# WV-02 egui共存

## 目的

egui UI と WebView の共存可能性を確認する。

## 実施結果

### PoC-2e Dock配置確認

確認項目:

* Dock移動
* Dockリサイズ
* Dockタブ切替
* Floating Panel
* WebView再配置
* 入力競合

確認結果:

* Dock移動成功
* Dockリサイズ成功
* Dockタブ切替成功
* WebView Child Window追従成功
* WebViewリサイズ追従成功
* WebViewタブ非表示時 Hide 成功
* 他タブの WebView Panel へのドラッグ成功
* Dock操作と入力競合は解消

#### 確認された課題

* Floating Panel 表示時にネイティブサーフェスとのZオーダー競合が発生する
* 本課題は WebView 固有ではなく GPU Viewport 等のネイティブサーフェス全般に共通する

#### 判定

条件付き成功

以下を確認した。

- egui_dock と WebView の共存成立
- Dock移動成功
- Dockリサイズ成功
- Dockタブ切替成功
- WebView Child Window追従成功
- WebViewリサイズ追従成功
- Dock操作との入力競合解消

ただし Child Window Overlay 方式では
Floating Panel とネイティブサーフェス間の
Zオーダー課題が残る。

### PoC-2f フローティング禁止方式評価

目的:

Floating Panel を禁止することで
Child Window Overlay 方式の
Zオーダー課題を回避可能か確認する。

確認事項:

- フローティング無効化可否
- Dock移動への影響
- UXへの影響
- GPU Viewportへの適用可能性

#### 確認された課題
なし

#### 判定
成功

#### 設計メモ

PoC-2f により、egui_dock では TabViewer の `allowed_in_windows()` を利用することで Floating Panel を禁止できることを確認した。

本検証で確認された Floating Panel とネイティブサーフェス間の Z オーダー課題は、WebView 固有の問題ではなく、Child Window Overlay 方式を利用するネイティブサーフェス全般に共通する課題である。

対象例:

* WebView
* GPU Viewport
* Video Surface
* DirectX Surface
* Vulkan Surface

そのため Workflow IDE Framework では以下の設計方針を候補とする。

* 通常の egui Panel は Floating Panel を許可する
* ネイティブサーフェスを含む Panel を使用する場合は Panel を Dock 内のみ許可する

期待効果:

* Floating Panel とネイティブサーフェス間の Z オーダー競合回避
* Child Window Overlay 方式の継続利用
* GPU Viewport 等への横展開
* UI 操作性の維持

## 次工程

### WV-03 複数WebView (WebView + GPU Viewport)

確認事項:

- WebView + GPU Viewport
- 複数ネイティブサーフェス配置

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [WebView 検証結果](./WebView_検証結果.md) > WV-02 egui共存
