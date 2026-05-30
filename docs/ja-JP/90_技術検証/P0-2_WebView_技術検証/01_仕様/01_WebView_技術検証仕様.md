<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009601Z-R8M7
lang: ja-JP
canonical_title: P0-2 WebView 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 技術検証仕様

# WebView 技術検証仕様

# 1. 概要

wry を利用した WebView Support Panel の成立性を確認する。

# 2. 検証対象

- wry
- egui coexist
- Window coexist

# 3. 確認事項

## 3.1 WebView 表示

WebView を表示可能であること。

## 3.2 egui coexist

WebView と egui を共存可能であること。

## 3.3 Multi Window

複数 Window を扱えること。

## 3.4 Command Bridge

WebView から IDE Command を呼び出せる構造を確認する。

## 3.5 Focus

WebView と egui 間で Focus 切替可能であること。

# 4. 今後の拡張

- JavaScript Bridge
- IDE Control
- AI Chat
- Help System

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 技術検証仕様