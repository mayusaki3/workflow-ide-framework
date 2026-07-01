<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: Custom Title Bar 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1e Custom Title Bar 技術検証](../README.md) > Custom Title Bar 技術検証仕様

# Custom Title Bar 技術検証仕様

# 1. 概要

Linux native window title 日本語問題を回避するため、renderer 内で custom title bar を実装可能か確認する。

# 2. 検証目的

- native title 非依存化
- 日本語 title 表示
- Linux/Windows 共通 UI
- Runtime IDE への適用性確認

# 3. 検証対象

- undecorated window
- egui custom title bar
- drag move
- close button
- minimize button
- maximize button

# 4. 要求事項

- EmbeddedFont と共存可能
- Linux fallback と共存可能
- 日本語 title 表示可能
- Runtime IDE へ流用可能

# 5. 想定結果

renderer 内 title 表示により Linux native title 日本語問題を回避できる。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1e Custom Title Bar 技術検証](../README.md) > Custom Title Bar 技術検証仕様