# 第2章 Application-level Public I/F 設計

## 1. 目的

Kairi 機能仕様 第2章を Consumer Application から単独で利用可能な境界にするため、現行実装で不足している Application-level Public I/F を確定する。

対象は次の4項目である。

1. Menu / Command
2. Status
3. Application lifecycle Event
4. Project lifecycle Event

Workspace / Panel 固有 I/F は第3章以降で扱う。

## 2. 設計原則

- Kairi 標準 Menu / Project lifecycle を Consumer が再実装しなくてよいこと。
- Consumer 固有機能は Kairi の内部 `FrameworkHost` を直接操作せず登録できること。
- 表示定義と処理を分離すること。
- Event は「状態が変わった事実」の通知とし、処理を横取りする interceptor としないこと。
- dirty confirmation や Project compatibility 等、既存 Project lifecycle の判断規則を Event で迂回できないこと。
- 第3章 Workspace に依存しないこと。

## 3. Menu / Command

### 3.1 Command

Consumer が実行可能な操作を Command として登録する。

初期 I/F は次を必要とする。

- stable Command ID
- 表示 label
- optional shortcut 表示
- enabled state
- execution callback

Command ID は Application 内で一意とする。

### 3.2 Menu

Menu は Command の表示場所を定義する。

初期実装では次を許可する。

- Kairi 標準 File / Help Menu への Consumer Command 追加
- Consumer 独自 top-level Menu の追加

Kairi 標準 Project Command 自体の置換は第2章の初期 I/F では許可しない。

### 3.3 責務

Kairi:
- Menu rendering
- enabled state の反映
- Command dispatch

Consumer:
- Command の意味
- Command callback 内の Application 固有処理

## 4. Status

Status は Application 全体から確認できる軽量な状態表示とする。

初期 I/F は次を持つ。

- stable Status ID
- label
- value
- visibility

Status は通知履歴やログの代替ではない。長時間保持する Application state の簡易表示を対象とする。

Consumer は登録済み Status item の value / visibility を更新できる必要がある。

## 5. Application lifecycle Event

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

## 6. Project lifecycle Event

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

## 7. Event payload

Project Event は少なくとも次の情報を Application が識別できる形にする。

- event kind
- Project root（保存済み Project の場合）
- Project name（取得可能な場合）

未保存 Project は Project root を持たない。

Application Event / Project Event とも callback 実行中に Kairi の内部可変 state を直接公開しない。

## 8. API配置

Public I/F は責務別 module に分離する。

```text
src/
├─ command.rs
├─ status.rs
├─ application_event.rs
└─ project_event.rs
```

`lib.rs` は Application builder からこれらを登録する入口を提供する。

## 9. テスト方針

最低限次を自動テストする。

### Command
- Command ID を保持できる
- enabled=false の Command は dispatch されない
- ID 重複を検出できる

### Status
- Status item 登録
- value 更新
- visibility 更新
- ID 重複を検出できる

### Application Event
- callback に Event kind が渡る
- Started は一度だけ発火する

### Project Event
- New 成功時 Created
- Open 成功時 Opened
- decision pending 中は Opened を発火しない
- Save 成功時 Saved
- Save As 成功時 SavedAs
- dirty confirmation 前後で CloseRequested / Closed を区別
- failure 時に成功 Event を発火しない

## 10. 実装順序

1. Command model / registry
2. Status model / registry
3. Event model
4. Application builder API
5. FrameworkHost 統合
6. Project lifecycle 統合
7. sample_minimum_application で第2章のみの利用例を確認
8. Reference を実装済み I/F に更新

この完了をもって、第2章 Application / Project を Consumer Application に対して「利用可能」と扱える状態とする。
