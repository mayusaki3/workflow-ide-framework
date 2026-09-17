[日本語](./README.md) | [English](./README_en-US.md)

# Workflow IDE Framework

Workflow IDE Framework は、IDE 型アプリケーションを構築するための Rust ベース IDE Platform Framework です。  
Windows / macOS / Linux 上で動作する、Dock 型・Scene 型 UI を持つ IDE アプリケーション構築を目的とします。

## 開発状態

Phase 0 の初期技術検証を完了し、検証結果を `main` へ統合する段階に到達しました。現在は最初の開発版 **v0.1.0** の仕様・実装作業を `develop` ブランチで開始しています。

初期検証で未検証または残課題となった OS / IME / GPU Driver 等の項目は記録を保持し、v0.1.0 以降の実装・検証で継続して扱います。

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

- [目次](./docs/ja-JP/目次.md)

## 採用仕様

ドキュメントは HLDocS を採用します。

- https://github.com/mayusaki3/HLDocS

---
