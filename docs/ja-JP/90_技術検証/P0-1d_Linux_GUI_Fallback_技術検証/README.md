<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260528-010201Z-G8W3
lang: ja-JP
canonical_title: P0-1d Linux GUI Fallback 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1d Linux GUI Fallback 技術検証

# P0-1d Linux GUI Fallback 技術検証

# 1. 目的

Linux GUI backend 問題に対する fallback 運用方針を検証する。

# 2. 関連ドキュメント

## 2.1 背景

- [P0-1c Linux Native Window Title 技術検証](../P0-1c_Linux_Native_Window_Title_技術検証/README.md)

## 2.2 検証仕様

- [Linux GUI Fallback 検証ケース](./02_検証仕様/01_検証ケース.md)

# 3. 検証対象

本検証は、P0-1d 独自の Cargo project を実行するものではない。

P0-1c の Cargo project を Linux fallback 条件付きで起動し、fallback 起動方式の有効性を検証する。

検証対象 project:

```text
../P0-1c_Linux_Native_Window_Title_技術検証
```

検証対象:

- software renderer fallback
- X11 fallback
- Hyper-V Ubuntu Desktop
- GUI backend stability
- Runtime 起動方針

# 4. 想定 fallback

```bash
LIBGL_ALWAYS_SOFTWARE=1
WINIT_UNIX_BACKEND=x11
```

# 5. 実行方法
...(省略同内容)...

# 9. P0-1e 引継ぎ

P0-1d で Linux GUI backend 起動安定化方針を確認後、次段階として Custom Title Bar 実装可能性を検証する。

後続:

- ../P0-1e_Custom_Title_Bar_技術検証/README.md

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1d Linux GUI Fallback 技術検証