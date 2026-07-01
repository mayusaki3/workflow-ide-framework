<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000001Z-WV00
lang: ja-JP
canonical_title: WV-00 Dock埋め込み成立性確認
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-00 Dock埋め込み成立性確認

# WV-00 Dock埋め込み成立性確認

## 目的

WebView 技術検証を開始する前に、egui_dock と WebView の組み合わせが Support Panel として成立するか確認する。  
まずは Windows を対象とし、その後 Linux / macOS での成立確認に進む。

本検証は WV-01 以降の前提条件とする。

## 前提条件

### 判定対象

#### 案A

egui_dock + wry Native View

#### 案B

egui_dock + Child Window + wry

#### 案C

egui_dock + HTML Renderer

## PoC

### 検証内容

#### WV-00-01

Dock Panel の矩形取得が可能であること。

#### WV-00-02

Dock Panel の移動を検知できること。

#### WV-00-03

Dock Panel のリサイズを検知できること。

#### WV-00-04

WebView 表示領域を Dock Panel に追従させられること。

#### WV-00-05

Dock 移動後も WebView が正しい位置に表示されること。

#### PoC-0 Dock矩形取得

実施内容

1. egui_dock 導入
2. Dock生成
3. テストPanel生成
4. Panel矩形取得
5. Panel移動検知
6. Panelリサイズ検知

確認項目

- Panel矩形取得可能
- Panel移動検知可能
- Panelリサイズ検知可能
- 毎フレーム矩形更新可能

### 実施結果

完了

- WV-00-01 Dock Panel矩形取得
- WV-00-02 Dock移動検知
- WV-00-03 Dockリサイズ検知
- WV-00-04 Child Window配置およびDock追従
- WV-00-05 Dock再配置後の追従

確認結果

- Dock操作正常
- Child Window正常復帰

確認された課題

- Child Window が Dock UI 操作を阻害する

回避策

- Dock操作中のみ Child Window を非表示
- Dock操作終了後に Child Window を再表示
- ここでのDock操作は、Child Windowが配置されたDock Panelにタブをドラッグする操作を指す。

### 評価

PoC-0 により Dock Panel の矩形取得、移動検知、リサイズ検知が成立した。

また Child Window Overlay 方式により Dock Panel への追従も成立した。

Dock UI 操作時の入力競合は、Dock 操作中のみ Child Window を非表示とする方式で解消可能であることを確認した。

### 後続検証

- WV-01 WebView表示

## WV評価

### 判定

条件付き成功

### 根拠

案A

不成立

案B

成立

確認事項

- Dock移動追従成功
- Dockリサイズ追従成功
- Dock再配置後追従成功
- Child Window Overlay方式成立
- Dock UIとの共存制御成立

制約

- Dock UIとの共存制御が必要

案C

未評価

## 次工程

### WV-01 WebView表示

確認事項

- WebView生成
- URL表示
- build_as_child方式成立性確認

## 備考

WV-00 は P0-2 WebView 技術検証全体の Go / No-Go 判定とする。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-00 Dock埋め込み成立性確認
