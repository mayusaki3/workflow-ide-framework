<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000002Z-WV01
lang: ja-JP
canonical_title: WV-01 WebView表示
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-01 WebView表示

# WV-01 WebView表示

## 目的

単一 WebView の生成および表示が成立することを確認する。

本検証では、WV-00 にて成立した Child Window Overlay 方式を前提とし、wry を利用した WebView 表示が可能であることを確認する。

まずは Windows を対象として成立性を確認し、その結果を基に Linux および macOS で同一検証を実施する。

## 前提条件

### WV-00

WV-00 により、Dock 埋め込み成立性確認は条件付き成功と判定済みである。

確認済み事項

- Dock Panel矩形取得
- Dock移動検知
- Dockリサイズ検知
- Child Window Overlay方式
- Dock追従制御

### 前提PoC

#### PoC-1c 親Window取得

目的

eframe 0.33 から winit::window::Window を取得可能か確認する。

成功条件

- Window Handle を取得できる
- Window Position を取得できる
- Window Size を取得できる

#### PoC-1d winit Window生成

目的

winit を直接利用して Window を生成可能か確認する。

確認事項

- winit EventLoop
- 複数 Window生成
- EventLoop競合有無

## PoC

### 検証内容

#### PoC-2a wry導入確認

確認内容

- Cargo.toml
- wry 0.53.5
- webview2-com

#### PoC-2b build_as_child調査

確認内容

- WebViewBuilder
- build_as_child

#### PoC-2c WebViewBuilder調査

確認内容

- WebViewBuilder::new()
- with_url()

#### PoC-2d HasWindowHandle調査

確認内容

- build_as_child()
- build()
- eframe::Frame 利用可否
- WebView生成可否

### 実施結果

#### PoC-1c 親Window取得

確認結果

- Window Handle取得可能
- Window Position取得可能
- Window Size取得可能

判定

成功

#### PoC-1d winit Window生成

確認結果

- 主Window生成成功
- 追加Window生成成功
- EventLoop競合なし

判定

成功

#### PoC-2a wry導入確認

確認結果

Windows

- cargo build 成功
- wry 導入成功
- webview2-com 導入成功
- eframe 0.33.0 と共存可能

判定

成功

#### PoC-2b build_as_child調査

確認結果

Windows

- build_as_child 存在確認
- HWND は HasWindowHandle を実装していない

判定

HWND 直結方式は不採用

#### PoC-2c WebViewBuilder調査

確認結果

Windows

- WebViewBuilder::new() 成功
- with_url("https://example.com") 成功
- cargo run 成功

判定

成功

#### PoC-2d HasWindowHandle調査

確認結果

Windows

- build_as_child(frame) 成功
- WebView生成成功
- example.com表示成功
- アプリケーション異常終了なし

判定

成功

### 評価

WV-01 の目的である WebView 表示は成立した。

特に build_as_child(frame) を利用することで、eframe と wry の共存が可能であることを確認した。

一方で Child Window HWND を直接親として利用する方式は採用できないことが判明した。

### 後続検証

WV-02 egui共存

## WV評価

### 判定

成功

### 根拠

確認済み事項

- WebView生成成功
- example.com表示成功
- build_as_child(frame) 成功
- アプリケーション異常終了なし
- eframe と wry の共存成功

### 制約

- HWND を直接利用できない
- build_as_child(frame) は Root Window 基準で生成される

## 次工程

### WV-02 egui共存

確認事項

- Dock移動
- Dockリサイズ
- Dockタブ切替
- Child Window追従
- WebView表示維持
- 入力競合

## 備考

WV-01 により WebView の生成および表示は成立した。

次工程では、実際の Workflow IDE Framework を想定し、egui_dock と WebView の共存を確認する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-01 WebView表示
