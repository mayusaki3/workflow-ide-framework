<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260528-010101Z-F7P4
lang: ja-JP
canonical_title: Linux Native Window Title 検証結果
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../README.md) > Linux Native Window Title 検証結果

# Linux Native Window Title 検証結果

# 1. 状態

PASS。

# 2. 検証対象

P0-1c は、Font download なしの Linux native title 検証である。

P0-1a / P0-1b で確認した renderer 内日本語表示は、本検証の主判定対象に含めない。

# 3. 検証条件

## 3.1 Font 条件

cleanup script により EmbeddedFont asset を削除した状態で実行する。

```bash
./scripts/cleanup_fonts.sh
cargo run
```

## 3.2 環境

```text
LANG=ja_JP.UTF-8
XDG_SESSION_TYPE=wayland
```

# 4. 想定していた結果

Linux native window title の日本語表示が成立する。

# 5. 実際の結果

## 5.1 native title

### ASCII

ASCII title は正常表示。

### 日本語

日本語部分は □ 化した。

### mixed

mixed title でも日本語部分のみ □ 化した。

## 5.2 renderer 内日本語表示

Font download なし条件では renderer 内日本語表示も □ 化した。

ただし、renderer 内日本語表示は P0-1a / P0-1b の検証対象であり、P0-1c の主判定対象ではない。

# 6. 判定

Linux native title の日本語表示は未成立。

Font download なし条件で Linux native title 日本語問題を再現できた。

# 7. 結論

P0-1c では、Linux native window title の日本語表示が成立しないことを確認した。

次段では、EmbeddedFont あり条件でも native title が改善するかを P0-1d で確認する。

# 8. 引継ぎ先

以下で継続検証する。

- [P0-1d Linux GUI Fallback 技術検証](../../P0-1d_Linux_GUI_Fallback_技術検証/README.md)

# 9. 検出事項

## 9.1 native title と renderer は別系統

native title は OS / window manager / toolkit integration 側の表示であり、renderer 内表示と分離して扱う必要がある。

## 9.2 Font なし条件の固定

cleanup script により、P0-1c 単独検証を Font なし条件へ戻せる。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../README.md) > Linux Native Window Title 検証結果