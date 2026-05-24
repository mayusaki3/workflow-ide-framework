<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001801Z-J8L2
lang: ja-JP
canonical_title: Plugin アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Plugin アーキテクチャー

# Plugin アーキテクチャー

# 1. 概要

Workflow IDE Framework は IPC Plugin 構造を採用する。

# 2. Plugin 構造

Plugin は独立 process として動作可能とする。

IPC により IDE Core と通信する。

# 3. Plugin 用途

- Runtime 拡張
- Workflow Node
- Tool
- Support 機能

# 4. 分離

Plugin は IDE Core と疎結合構造とする。

# 5. 今後の詳細仕様

- Plugin Lifecycle
- Plugin IPC
- Plugin Permission
- Plugin API
- Plugin Discovery

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Plugin アーキテクチャー