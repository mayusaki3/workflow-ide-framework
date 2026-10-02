# Application-level Public I/F テスト仕様

## 対象

Kairi 機能仕様 第2章 Application / Project のうち、Menu / Command、Status、Application lifecycle Event、Project lifecycle Event の Public I/F。

## Command

| ID | 条件 | 期待結果 |
|---|---|---|
| CMD-01 | 一意なIDでCommand登録 | 登録成功 |
| CMD-02 | 同一IDを再登録 | 重複として拒否 |
| CMD-03 | enabled=true | dispatchでcallback実行 |
| CMD-04 | enabled=false | dispatchでcallback非実行 |

## Status

| ID | 条件 | 期待結果 |
|---|---|---|
| STS-01 | 一意なIDでStatus登録 | 登録成功 |
| STS-02 | 同一IDを再登録 | 重複として拒否 |
| STS-03 | value更新 | 対象itemのみ更新 |
| STS-04 | visibility更新 | 対象itemのみ更新 |
| STS-05 | 未登録ID更新 | not-foundを返す |

## Application Event

| ID | 条件 | 期待結果 |
|---|---|---|
| APP-EVT-01 | Starting発火 | callbackへStarting |
| APP-EVT-02 | Started発火 | callbackへStarted |
| APP-EVT-03 | Started処理を複数frame実行 | Startedは1回のみ |
| APP-EVT-04 | 終了要求 | CloseRequested |
| APP-EVT-05 | 終了確定 | Closing |

## Project Event

| ID | 条件 | 期待結果 |
|---|---|---|
| PRJ-EVT-01 | New成功 | Created |
| PRJ-EVT-02 | Open成功 | Opened |
| PRJ-EVT-03 | Openがuser decision待ち | Openedなし |
| PRJ-EVT-04 | decision受理後Open成功 | Opened |
| PRJ-EVT-05 | Save成功 | Saved |
| PRJ-EVT-06 | Save As成功 | SavedAs |
| PRJ-EVT-07 | Close要求 | CloseRequested |
| PRJ-EVT-08 | dirty confirmation待ち | Closedなし |
| PRJ-EVT-09 | Close確定 | Closed |
| PRJ-EVT-10 | lifecycle operation失敗 | 対応する成功Eventなし |

## 完了条件

- 新規自動テストがすべて成功すること。
- 既存 Project / Resource 系テストに regression がないこと。
- `sample_minimum_application` が第2章の機能だけで構成可能であること。
