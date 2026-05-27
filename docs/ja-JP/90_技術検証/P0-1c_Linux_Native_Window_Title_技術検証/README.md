<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-010701Z-C9T4
lang: ja-JP
canonical_title: P0-1c Linux Native Window Title 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1c Linux Native Window Title 技術検証

# P0-1c Linux Native Window Title 技術検証

# 1. 目的

Linux 環境で native window title の日本語表示が成立するか確認する。

# 2. 関連ドキュメント

## 2.1 仕様

- [Linux Native Window Title 技術検証仕様](./01_仕様/01_Linux_Native_Window_Title_技術検証仕様.md)

## 2.2 検証仕様

- [Linux Native Window Title 検証ケース](./02_検証仕様/01_検証ケース.md)

## 2.3 検証結果

- [Linux Native Window Title 検証結果](./03_検証結果/README.md)

# 3. 背景

P0-1b にて IDE renderer 内の日本語表示は成立した。

しかし Linux native window title のみ文字化けが発生した。

# 4. 想定原因

- Wayland
- X11
- GTK
- locale
- fontconfig
- window manager

# 5. 検証対象

- native title
- locale
- GTK
- eframe
- window manager
- Wayland/X11 差異

# 6. 実行方法

## 6.1 Linux

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

cargo run
```

# 7. 確認項目

以下を確認する。

- native window title
- LANG
- LC_ALL
- XDG_SESSION_TYPE
- Wayland/X11
- ASCII / 日本語 / mixed title

# 8. 想定方針

本問題は Runtime IDE renderer 問題ではなく、OS integration 問題として扱う。

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1c Linux Native Window Title 技術検証