<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261006-kairi-application-public-if-design
lang: ja-JP
canonical_title: Application-level Public I/F 設計
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > 仕様 > Application / Project > Application-level Public I/F設計

# 第2章 Application-level Public I/F 設計

## 1. 目的

Kairi 機能仕様 第2章を Consumer Application から単独で利用可能な境界にするため、現行実装で不足している Application-level Public I/F を確定する。

対象は次の4項目である。

1. Menu / Command
2. Status
3. Application lifecycle Event
4. Project lifecycle Event

Workspace / Panel 固有 I/F は第3章以降で扱う。

## 2. Menu / Command

### 2.1 Command

Consumer が実行可能な操作を Command として登録する。

初期 I/F は次を必要とする。

- stable Command ID
- 表示 label
- optional shortcut 表示
- enabled state
- execution callback

Command ID は Application 内で一意とする。

### 2.2 Menu

Menu は Command の表示場所を定義する。

初期実装では次を許可する。

- Kairi 標準 File / Help Menu への Consumer Command 追加
- Consumer 独自 top-level Menu の追加

Kairi 標準 Project Command 自体の置換は第2章の初期 I/F では許可しない。

### 2.3 責務

Kairi:
- Menu rendering
- enabled state の反映
- Command dispatch

Consumer:
- Command の意味
- Command callback 内の Application 固有処理

## 3. Status

Status は Application 全体から確認できる軽量な状態表示とする。

初期 I/F は次を持つ。

- stable Status ID
- label
- value
- visibility

Status は通知履歴やログの代替ではない。長時間保持する Application state の簡易表示を対象とする。

Consumer は登録済み Status item の value / visibility を更新できる必要がある。

## 4. Application lifecycle Event

初期 Event は次とする。

- `Starting`
- `Started`
- `CloseRequested`
- `Closing`

意味:

- Starting: Kairi host の起動処理開始
- Started: host 初期化完了後、UI loop が利用可能になった最初の時点
- CloseRequested: User / OS による終了要求を Kairi が処理するとき
- Closing: dirty Project 等の確認を通過し、終了を確定したとき

Event は通知であり、Consumer が Kairi の終了判定を上書きする veto I/F とはしない。

eframe lifecycle 上、厳密な発火位置は実装時に Reference へ記録する。

## 5. Project lifecycle Event

初期 Event は次とする。

- `Created`
- `Opened`
- `Saved`
- `SavedAs`
- `CloseRequested`
- `Closed`

Project operation が失敗した場合、成功 Event は発火しない。

Open で User decision / recovery が必要な場合、`Opened` は `accept_open()` が成功した後に発火する。

CloseRequested は dirty confirmation 前に通知できるが、Closed は実際に Project session が破棄された後だけ発火する。

## 6. Event payload

Project Event は少なくとも次の情報を Application が識別できる形にする。

- event kind
- Project root（保存済み Project の場合）
- Project name（取得可能な場合）

未保存 Project は Project root を持たない。

Application Event / Project Event とも callback 実行中に Kairi の内部可変 state を直接公開しない。

## 7. API配置

Public I/F は責務別 module に分離する。

```text
src/
├─ command.rs
├─ status.rs
├─ application_event.rs
└─ project_event.rs
```

`lib.rs` は Application builder からこれらを登録する入口を提供する。

---

[目次](../目次.md) > 仕様 > Application / Project > Application-level Public I/F設計
