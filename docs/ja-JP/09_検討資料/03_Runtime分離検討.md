[目次](../目次.md) > docs/ja-JP > 09_検討資料 > Runtime 分離検討

# Runtime 分離検討

## 概要

UI と Runtime は process 分離可能な構成を考慮する。

## 目的

- Runtime 性能最大化
- UI 負荷分離
- build 分離
- crash 分離
- remote runtime 対応

## 想定構成

```text
UI Process
  ├─ Dock UI
  ├─ Page
  ├─ Scene
  └─ UI State

IPC Boundary

Runtime Process
  ├─ Runtime
  ├─ Build
  ├─ GPU Processing
  ├─ Simulation
  └─ Worker
```

## UI 連携

Page 間連携は UI 内 Event Bus を利用する。

## Runtime 連携

UI と Runtime は UI API を介して連携する。

---
[目次](../目次.md) > docs/ja-JP > 09_検討資料 > Runtime 分離検討