<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001201Z-C8M2
lang: ja-JP
canonical_title: Event State アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Event State アーキテクチャー

# Event / State アーキテクチャー

# 1. 概要

Workflow IDE Framework は Event / State モデルを採用する。  
State は現在状態を表し、Event は状態変化通知を表す。

# 2. Event

Event は Runtime・Workflow・UI・Plugin などの状態変化通知として扱う。

例:

- WorkflowStarted
- NodeStarted
- NodeFinished
- RuntimeConnected
- RuntimeDisconnected
- StateChanged
- CommandExecuted

# 3. State

State は Runtime・Workflow・UI の現在状態を保持する。  
State Store は Redux 型構造を採用する。

# 4. Event Bus

Event Bus は async channel ベースとする。  
publish / subscribe により Event を通知する。

# 5. 更新モデル

基本更新フローは以下とする。

Command
↓
State 更新
↓
Event 発行
↓
UI 更新

# 6. 今後の詳細仕様

詳細仕様は必要タイミングで分割する。

例:

- Event 定義
- State Store
- Reducer
- UI 部品
- Runtime Event
- Scheduler

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > Event State アーキテクチャー