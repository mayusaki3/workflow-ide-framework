<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001301Z-D2K4
lang: ja-JP
canonical_title: Command アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Command アーキテクチャー

# Command アーキテクチャー

# 1. 概要

Workflow IDE Framework は Command モデルを採用する。  
UI・API・WebView・外部 Runtime などの操作を統一 Command として扱う。

# 2. Command モデル

Command は IDE 操作単位として扱う。

例:

- OpenScene
- StartWorkflow
- StopWorkflow
- OpenViewport
- CloseViewport

# 3. Undo / Redo

Undo / Redo を考慮する。

# 4. Event / State 連携

Command 実行後は State 更新および Event 発行を行う。

# 5. 今後の詳細仕様

- Command Lifecycle
- Command History
- Undo / Redo
- Command Permission
- WebView Command Bridge

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Command アーキテクチャー