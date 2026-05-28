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

# 2. 想定していた結果

Linux native window title の日本語表示が成立する。

# 3. 実際の結果

## 3.1 locale

### 結果

```text
LANG=ja_JP.UTF-8
```

### 判定

locale 設定自体は正常。

## 3.2 renderer backend

### 初期状態

以下 error 発生。

```text
libEGL
MESA
ZINK
```

### 判定

Hyper-V Ubuntu Desktop 環境では GPU backend 問題が発生した。

## 3.3 software renderer fallback

### 実施内容

```bash
LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 cargo run
```

### 結果

- window 表示成功
- egui renderer 正常
- Panel 日本語表示成功

### 判定

software renderer fallback により Linux GUI 実行が安定化した。

## 3.4 native title

### ASCII

ASCII title は正常表示。

### 日本語

日本語部分のみ □ 化。

### mixed

mixed title でも日本語部分のみ □ 化。

### 判定

Linux native title UTF-8 日本語表示問題を確認した。

# 4. 結論

本問題は egui renderer 問題ではない。

Linux native window title / OS integration 側問題である可能性が高い。

# 5. 許容判断

IDE renderer は成立しているため Runtime IDE blocker ではない。

継続検討課題として扱う。

# 6. 引継ぎ先

以下で継続検証する。

- [P0-1d Linux GUI Fallback 技術検証](../../P0-1d_Linux_GUI_Fallback_技術検証/README.md)

# 7. 検出事項

## 7.1 renderer と native title は別問題

Panel renderer と native title は別系統である。

## 7.2 Hyper-V Linux GUI

Hyper-V Ubuntu Desktop 環境では software renderer fallback が必要な可能性が高い。

## 7.3 UTF-8 title

Linux native title UTF-8 handling 問題の可能性がある。

# 8. 今後の候補

- GTK backend 詳細調査
- Wayland compositor 調査
- X11 専用検証
- eframe/winit upstream 調査
- native title workaround

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../README.md) > Linux Native Window Title 検証結果