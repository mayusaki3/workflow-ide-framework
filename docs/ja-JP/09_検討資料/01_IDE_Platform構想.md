[目次](../目次.md) > docs/ja-JP > 09_検討資料 > IDE Platform 構想

# IDE Platform 構想

## 概要

Workflow IDE Framework は、IDE 型アプリケーションを構築するための共通基盤を提供する。

本 framework は、単なる UI framework ではなく、Workspace Runtime Platform を目指す。

## 想定用途

- Runtime IDE
- Simulation IDE
- SansaVRM Studio AI
- AI Studio
- Asset Studio
- Workflow IDE

## 想定構成

```text
workflow-ide-framework
  ├─ Dock UI
  ├─ Scene
  ├─ Page
  ├─ Event Bus
  ├─ Command System
  ├─ UI API
  ├─ Runtime Connection
  ├─ GPU Viewport
  ├─ Workspace Persistence
  └─ Async Task
```

## 分離方針

framework は IDE の器を提供する。

Domain Model や Runtime 実装は application 側で実装する。

## Runtime 分離

UI と Runtime は process 分離可能な構成を考慮する。

## GPU Viewport

MuJoCo や VRM preview 等を埋め込み表示可能な GPU Viewport を考慮する。

---
[目次](../目次.md) > docs/ja-JP > 09_検討資料 > IDE Platform 構想