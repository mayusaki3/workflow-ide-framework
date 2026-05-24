<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001701Z-H4W9
lang: ja-JP
canonical_title: WebView アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > WebView アーキテクチャー

# WebView アーキテクチャー

# 1. 概要

Workflow IDE Framework は WebView を Support Panel として扱う。

IDE Core UI は egui ベース構造とする。

# 2. WebView 構造

WebView は Panel 単位で扱う。

wry を利用する。

# 3. 想定用途

- Help
- Documentation
- Tutorial
- AI Chat

# 4. IDE 連携

WebView から IDE 操作を可能とする。

IDE 操作は Command 経由で行う。

# 5. 今後の詳細仕様

- WebView Lifecycle
- WebView Command Bridge
- Focus 管理
- Permission
- Sandbox

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > WebView アーキテクチャー