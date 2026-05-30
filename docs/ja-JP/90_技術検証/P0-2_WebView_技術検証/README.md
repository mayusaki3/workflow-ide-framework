<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009501Z-Q4K1
lang: ja-JP
canonical_title: P0-2 WebView 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-2 WebView 技術検証

# P0-2 WebView 技術検証

# 1. 目的

wry を利用した WebView Support Panel 構造の成立性を確認する。

# 2. 検証項目

- WebView 表示
- egui coexist
- Command Bridge
- Multi Panel
- Window coexist
- Focus 切替

# 3. ドキュメント構成

## 仕様

* [WebView 技術検証仕様](01_仕様/01_WebView_技術検証仕様.md)

## 検証仕様

* [WebView 検証項目](02_検証仕様/WebView_検証項目.md)
* [WebView 検証ケース](02_検証仕様/01_検証ケース.md)
* [WV-00 Dock埋め込み成立性確認](02_検証仕様/WV-00_Dock埋め込み成立性確認.md)

## 検証結果

* [WebView 検証結果](03_検証結果/WebView_検証結果.md)

## 技術検証資料

* [PoC-0 Dock矩形取得](04_PoC/PoC-0_Dock矩形取得.md)
* [PoC-1c 親Window取得](04_PoC/PoC-1c_親Window取得.md)
* [PoC-1d winit Window生成](04_PoC/PoC-1d_winit_Window生成.md)

# 4. 現在の進捗

## 完了

* PoC-0 Dock矩形取得
* PoC-1a ViewportInfo調査
* PoC-1b FrameInfo調査
* PoC-1c 親Window取得調査

## 実施中

* WV-00 Dock埋め込み成立性確認

## 次工程

* PoC-1d winit Window生成
* PoC-1e Child Window生成
* PoC-1f Dock追従確認
* WV-01 WebView表示

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-2 WebView 技術検証