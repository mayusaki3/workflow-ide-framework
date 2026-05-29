<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000004Z-WV23
lang: ja-JP
canonical_title: WebView PoC設計
document_type: design
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView PoC設計

# WebView PoC設計

## 方針

P0-2では方式Bを採用する。

```text
Runtime IDE
├─ Dock Area
│  ├─ Explorer Panel
│  ├─ Properties Panel
│  └─ WebView Panel
├─ Viewport
└─ Command System
```

WebViewは独立ウィンドウではなく、Dock管理対象のSupport Panelとして扱う。

## PoC対象

### Phase 1

- WV-01 WebView表示
- WV-02 egui共存

### Phase 2

- WV-03 Multi Panel
- WV-04 Focus切替

### Phase 3

- WV-05 Rust→WebView
- WV-06 WebView→Rust

### Phase 4

- WV-07 Command Bridge

## Command Bridge構造

```text
WebView
 ↓
Bridge Layer
 ↓
Command Dispatcher
 ↓
Event Bus
```

WebViewはCommand Dispatcher以外へ直接アクセスしない。

## PoC完了条件

- WV-01～WV-07成功
- Dock内WebView構造成立
- Help Panel実装可能と判断できる
- AI Chat Panel実装可能と判断できる

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView PoC設計
