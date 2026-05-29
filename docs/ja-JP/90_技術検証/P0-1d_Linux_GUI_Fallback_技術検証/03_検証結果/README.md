<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260529-010101Z-J6R2
lang: ja-JP
canonical_title: Linux GUI Fallback 検証結果
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1d Linux GUI Fallback 技術検証](../README.md) > Linux GUI Fallback 検証結果

# Linux GUI Fallback 検証結果

# 1. 状態

記録待ち。

# 2. 検証対象

P0-1d は、P0-1c の Cargo project を Linux fallback 条件付きで実行し、GUI backend 起動安定化が成立するかを確認する検証である。

本検証では、P0-1d 独自の Cargo project は作成しない。

# 3. 検証条件

# 3.1 対象 project

```text
../P0-1c_Linux_Native_Window_Title_技術検証
```

# 3.2 fallback 条件

```text
LIBGL_ALWAYS_SOFTWARE=1
WINIT_UNIX_BACKEND=x11
```

# 3.3 Font 条件

P0-1d では、P0-1c 配下に EmbeddedFont asset を配置した状態で確認する。

# 4. 記録項目

以下を記録する。

- P0-1c fallback なし直接実行結果
- P0-1c fallback あり実行結果
- fallback により改善した項目
- fallback でも改善しない項目
- EmbeddedFont 有無による差異

# 5. 想定判定

Linux GUI backend failure 時は fallback 起動を許容する。

ただし、fallback は native window title 日本語表示を改善するためではなく、Linux GUI backend 起動安定化のための回避策として扱う。

# 6. 引継ぎ先

native window title 日本語問題が残存する場合、次段では native title 非依存の custom title bar を検証する。

- [P0-1e Custom Title Bar 技術検証](../../P0-1e_Custom_Title_Bar_技術検証/README.md)

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1d Linux GUI Fallback 技術検証](../README.md) > Linux GUI Fallback 検証結果