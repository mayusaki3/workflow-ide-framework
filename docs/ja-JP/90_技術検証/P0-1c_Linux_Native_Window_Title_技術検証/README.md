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

# 4. 検証条件

## 4.1 P0-1c 単独検証

P0-1c 単独検証では EmbeddedFont asset を使用しない。

OS default font 環境で native title を検証する。

検証前に cleanup script を実行する。

cleanup script:

```text
scripts/cleanup_fonts.sh
scripts/cleanup_fonts.ps1
```

## 4.2 P0-1d fallback 検証

P0-1d fallback 検証では、P0-1c 配下へ font download を実施可能とする。

font 配置先:

```text
assets/fonts/default
```

font setup script:

```text
scripts/setup_fonts.sh
scripts/setup_fonts.ps1
```

# 5. 想定原因

- Wayland
- X11
- GTK
- locale
- fontconfig
- window manager

# 6. 検証対象

- native title
- locale
- GTK
- eframe
- window manager
- Wayland/X11 差異

# 7. 実行方法

## 7.1 Linux

### P0-1c 単独検証

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

chmod +x ./scripts/cleanup_fonts.sh

./scripts/cleanup_fonts.sh

cargo run
```

### P0-1d fallback 用 font setup

通常の P0-1c 単独検証では実施しない。

P0-1d fallback 検証時のみ利用する。

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

chmod +x ./scripts/setup_fonts.sh

./scripts/setup_fonts.sh
```

# 8. 確認項目

以下を確認する。

- native window title
- LANG
- LC_ALL
- XDG_SESSION_TYPE
- Wayland/X11
- ASCII / 日本語 / mixed title

# 9. 想定方針

本問題は Runtime IDE renderer 問題ではなく、OS integration 問題として扱う。

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1c Linux Native Window Title 技術検証