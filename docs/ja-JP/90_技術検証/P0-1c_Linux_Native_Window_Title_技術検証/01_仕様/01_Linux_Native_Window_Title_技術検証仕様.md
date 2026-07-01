<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-010801Z-D4K8
lang: ja-JP
canonical_title: Linux Native Window Title 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../README.md) > Linux Native Window Title 技術検証仕様

# Linux Native Window Title 技術検証仕様

# 1. 概要

Linux 環境で native window title の日本語表示が成立するか確認する。

# 2. 背景

P0-1b にて Embedded Font による IDE renderer 内の日本語表示は成立した。

しかし native window title のみ Linux 環境で文字化けが発生した。

# 3. 検証目的

以下を切り分ける。

- eframe
- GTK
- Wayland
- X11
- locale
- fontconfig
- window manager

# 4. 検証対象

- native title
- locale
- LANG
- LC_ALL
- GTK backend
- Wayland/X11
- eframe window title

# 5. 想定結果

Linux native window title の日本語表示が成立する。

# 6. 許容条件

IDE renderer が成立しているため、本問題は Runtime IDE blocker ではない。

ただし UX 品質として継続検証する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../README.md) > Linux Native Window Title 技術検証仕様