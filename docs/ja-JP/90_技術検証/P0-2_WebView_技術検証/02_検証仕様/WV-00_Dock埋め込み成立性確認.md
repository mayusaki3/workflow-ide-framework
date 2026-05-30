<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000006Z-WV25
lang: ja-JP
canonical_title: WV-00 Dock埋め込み成立性確認
document_type: test_spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WV-00 Dock埋め込み成立性確認

# WV-00 Dock埋め込み成立性確認

## 目的

WebView 技術検証を開始する前に、egui_dock と WebView の組み合わせが Support Panel として成立するか確認する。

本検証は WV-01 以降の前提条件とする。

## 判定対象

### 案A

egui_dock + wry Native View

### 案B

egui_dock + Child Window + wry

### 案C

egui_dock + HTML Renderer

## 検証内容

### WV-00-01

Dock Panel の矩形取得が可能であること。

### WV-00-02

Dock Panel の移動を検知できること。

### WV-00-03

Dock Panel のリサイズを検知できること。

### WV-00-04

WebView 表示領域を Dock Panel に追従させられること。

### WV-00-05

Dock 移動後も WebView が正しい位置に表示されること。

## 成功条件

- Dock移動に追従する。
- Dockリサイズに追従する。
- Dock再配置後も表示が破綻しない。
- Focusが維持される。

## 判定

### A成立

案Aで成功条件を満たす。

次工程:

- WV-01開始
- WV-02開始

### A不成立 B成立

案Bへ移行。

次工程:

- WV-01開始
- WV-02開始

### A不成立 B不成立

案Cへ移行。

次工程:

- WebView検証仕様見直し

## 判定結果

### 完了

* WV-00-01 Dock Panel矩形取得
* WV-00-02 Dock移動検知
* WV-00-03 Dockリサイズ検知
* WV-00-04 Child Window配置およびDock追従
* WV-00-05 Dock再配置後の追従

### 確認された課題

* Child WindowがDock UI操作を阻害する

### 回避策

* Dock操作中のみChild Window非表示
* Dock操作終了後再表示

確認結果:

* Dock操作正常
* Child Window正常復帰

### 判定

案B:

egui_dock + Child Window + wry

成立

ただしDock UIとの共存制御が必要。

### 次工程

* WV-01 WebView表示

### 現時点の評価

案A:
不成立

案B:
成立（条件付き）

案C:
未評価

## 備考

WV-00はP0-2全体のGo/No-Go判定とする。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WV-00 Dock埋め込み成立性確認
