<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-010301Z-Y5L2
lang: ja-JP
canonical_title: EmbeddedFont 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1b EmbeddedFont 技術検証](../README.md) > EmbeddedFont 技術検証仕様

# EmbeddedFont 技術検証仕様

# 1. 概要

IDE renderer へ font asset を埋め込み、日本語表示を cross-platform 環境で統一可能か検証する。

# 2. 検証目的

- Linux / Windows 表示統一
- egui custom font
- Runtime font 構造
- 利用者 custom font
- Runtime font 切替

# 3. Embedded Font 構造

## 3.1 標準 font

Framework 側で標準 font を提供する。

## 3.2 custom font

利用者が独自 font を追加可能とする。

## 3.3 font selector

Workspace 単位で font 選択可能とする。

# 4. 想定 API

```rust
register_font()
set_default_font()
set_workspace_font()
reload_font()
```

# 5. 想定ディレクトリ

```text
assets/fonts/
 ├─ default/
 └─ custom/
```

# 6. 要求事項

## 6.1 cross-platform

固定 OS path を使用しない。

## 6.2 Runtime reload

Runtime 中に font reload 可能な構造を考慮する。

## 6.3 拡張性

icon font / emoji / multi language へ拡張可能とする。

# 7. 今後の検討

- font cache
- fallback chain
- variable font
- emoji color font

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1b EmbeddedFont 技術検証](../README.md) > EmbeddedFont 技術検証仕様