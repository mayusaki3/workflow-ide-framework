<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260529-000001Z-P0-1E
lang: ja-JP
canonical_title: P0-1e Custom Title Bar 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1e Custom Title Bar 技術検証

# P0-1e Custom Title Bar 技術検証

# 1. 目的

Linux native window title 日本語問題に対し、custom title bar により回避可能か確認する。

# 2. 関連ドキュメント

## 2.1 仕様

- [Custom Title Bar 技術検証仕様](./01_仕様/01_Custom_Title_Bar_技術検証仕様.md)

## 2.2 検証仕様

- [Custom Title Bar 検証ケース](./02_検証仕様/01_検証ケース.md)

## 2.3 検証結果

- [Custom Title Bar 検証結果](./03_検証結果/README.md)

## 2.4 引継ぎ元

- [P0-1d Linux GUI Fallback 技術検証](../P0-1d_Linux_GUI_Fallback_技術検証/README.md)

# 3. 背景

P0-1d にて以下を確認した。

- EmbeddedFont runtime load 成立
- renderer 内日本語表示成立
- Linux native title 日本語未成立
- fallback による libEGL/MESA/ZINK 回避成立

このため、native title 非依存構成を検証する。

# 4. 検証対象

- undecorated window
- egui custom title bar
- window drag
- close/minimize/maximize
- EmbeddedFont 共存
- Linux fallback 共存
- 日本語 title

# 5. 想定構成

```text
native title
↓
custom title bar
```

# 6. 想定結果

- renderer 内 title 日本語成立
- Linux native title 非依存
- Runtime IDE UI 共通化可能

# 7. 引継ぎ先候補

- P0-2 WebView 技術検証
- Runtime IDE

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1e Custom Title Bar 技術検証