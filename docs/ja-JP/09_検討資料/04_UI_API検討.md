[目次](../目次.md) > [検討資料目次](検討資料目次.md) > UI API 検討

# UI API 検討

## UI API

UI と Runtime / Project / Build 等の内部処理を分離するため、UI API を介して連携する。

## 目的

- UI 分離
- Runtime 分離
- 非同期処理分離
- remote runtime 対応
- headless runtime 対応

## UI API 対象

例：

- Project 操作
- Runtime 制御
- build 実行
- Runtime 状態取得
- ログ取得
- 設定取得

## Event Bus

Page 間連携は Event Bus を利用する。

## Runtime 非依存

UI は Runtime 内部実装へ直接依存しない。

---

[目次](../目次.md) > [検討資料目次](検討資料目次.md) > UI API 検討