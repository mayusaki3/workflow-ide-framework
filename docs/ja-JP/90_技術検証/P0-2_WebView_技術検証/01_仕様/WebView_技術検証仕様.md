<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000001Z-WV20
lang: ja-JP
canonical_title: WebView 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../P0-2_WebView_技術検証/README.md) > WebView 技術検証仕様

# WebView 技術検証仕様

## 目的

Workflow IDE Framework において、WebView を Support Panel として採用可能かを検証する。

本検証では、Rust + egui + wry 構成において以下を確認する。

- WebView の表示
- egui との共存
- 複数 WebView の利用
- Focus 切り替え
- Rust ⇔ JavaScript 通信
- IDE Command Bridge

## 対象構成

### IDE 構成

```text
Workflow IDE Framework
├─ egui
├─ GPU Viewport
├─ WebView (wry)
└─ Command System
```

### WebView の位置付け

WebView は IDE Core ではなく Support Panel として扱う。

利用用途例は以下とする。

- Help
- Tutorial
- Documentation
- AI Chat
- 外部 Web サービス連携

## 技術選定

| 項目 | 採用 |
|---|---|
| GUI | egui |
| Window Backend | eframe |
| WebView | wry |
| JavaScript Engine | WebView 標準機能 |
| 通信方式 | Command Bridge |

## 検証対象

### WebView 生成

単一 WebView を生成可能であること。

### egui 共存

egui と WebView が同一アプリケーション内で共存可能であること。

### Multi Panel

複数 WebView を同時生成可能であること。

### Focus 管理

以下の切り替えが可能であること。

- egui → WebView
- WebView → egui
- WebView A → WebView B

### Rust → JavaScript

Rust から JavaScript 関数を呼び出し可能であること。

### JavaScript → Rust

JavaScript から Rust 側イベントを発行可能であること。

### Command Bridge

JavaScript から IDE Command を呼び出し可能であること。

## Command Bridge 構想

### JavaScript

```javascript
window.ide.execute("sample_command", {
    value: 123
});
```

### Rust

```text
CommandDispatcher
  ↓
Command
  ↓
Event Bus
```

### 要求事項

WebView から IDE の内部実装へ直接アクセスしてはならない。

必ず Command 経由で連携すること。

## 成功条件

以下をすべて満たした場合、本検証を成功と判定する。

1. WebView 表示可能
2. egui 共存可能
3. 複数 WebView 生成可能
4. Focus 切替可能
5. Rust → JavaScript 通信可能
6. JavaScript → Rust 通信可能
7. Command Bridge 成立

## 次フェーズ

P0-2 完了後、以下を実施する。

- Help UI
- Tutorial UI
- AI Chat UI
- Documentation Viewer
- IDE Command 統合

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../P0-2_WebView_技術検証/README.md) > WebView 技術検証仕様
