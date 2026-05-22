[目次](docs/ja-JP/目次.md) > README

# Workflow IDE Framework

Workflow IDE Framework は、IDE 型アプリケーションを構築するための Rust ベース IDE Platform Framework です。

Windows / macOS / Linux 上で動作する、Dock 型・Scene 型 UI を持つ IDE アプリケーション構築を目的とします。

## 目的

以下のような IDE 型アプリケーションを構築するための共通基盤を提供します。

- Runtime IDE
- Simulation IDE
- AI Studio
- Asset Studio
- Workflow IDE
- GPU Viewport IDE

## 特徴

- Dock 型 UI
- Scene 切り替え
- Page ベース UI
- GPU Viewport
- Runtime 分離
- 非同期 Task
- Event Bus
- UI API
- Workspace / Session 保存
- Command System

## UI構造

```text
Page
  ↓
Tab
  ↓
Frame
  ↓
Scene
```

## 想定構成

```text
IDE Product
  ├─ workflow-ide-framework
  ├─ Runtime
  ├─ SDK
  ├─ Domain Model
  └─ Application Pages
```

## ドキュメント

- docs/ja-JP/目次.md
- docs/ja-JP/09_検討資料

## 採用仕様

ドキュメントは HLDocS を採用します。

- https://github.com/mayusaki3/HLDocS

---
[目次](docs/ja-JP/目次.md) > README