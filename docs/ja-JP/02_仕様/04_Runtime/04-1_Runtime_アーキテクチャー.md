<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001401Z-E5T8
lang: ja-JP
canonical_title: Runtime アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Runtime アーキテクチャー

# Runtime アーキテクチャー

# 1. 概要

Workflow IDE Framework は Runtime process 分離構造を採用する。  
UI と Runtime を分離し、外部 Runtime 接続を可能とする。

# 2. Runtime 構造

Runtime は独立 process として動作可能とする。  
UI process とは IPC を通じて通信する。

# 3. Runtime 接続

- Local Runtime
- Remote Runtime

を考慮する。

# 4. Event / State

Runtime 状態は Event / State モデルで扱う。

# 5. 今後の詳細仕様

- Runtime Lifecycle
- Runtime Scheduler
- Runtime State
- Runtime IPC
- Runtime Crash Recovery

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Runtime アーキテクチャー