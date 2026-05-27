<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-010201Z-X8R3
lang: ja-JP
canonical_title: P0-1b EmbeddedFont 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1b EmbeddedFont 技術検証

# P0-1b EmbeddedFont 技術検証

# 1. 目的

cross-platform 環境で、日本語表示を安定させる Embedded Font 構造を検証する。

# 2. 背景

P0-1a の検証により、OS font fallback のみでは Linux / Windows 間で表示差異が発生することを確認した。

そのため、IDE 側で font asset を保持し、UI renderer へ直接登録する構造を検証する。

# 3. 検証項目

- include_bytes! による font 埋め込み
- 日本語表示
- Linux / Windows 一致確認
- egui custom font
- Runtime 配布構造

# 4. 実行方法

## 4.1 Windows

### font setup

```powershell
cd C:\WORKPLACE\Makes\GitHub\workflow-ide-framework\docs\ja-JP\90_技術検証\P0-1b_EmbeddedFont_技術検証

powershell -ExecutionPolicy Bypass -File .\scripts\setup_fonts.ps1
```

### 実行

```powershell
cargo run
```

## 4.2 Linux

### font setup

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1b_EmbeddedFont_技術検証

chmod +x ./scripts/setup_fonts.sh

./scripts/setup_fonts.sh
```

### 実行

```bash
cargo run
```

# 5. 検討事項

## 5.1 利用者による font 選択

利用者が UI font を変更可能な構造を考慮する。

## 5.2 標準 font

Framework 標準 font を提供する。

## 5.3 カスタム font

利用者が独自 font を追加可能な構造を考慮する。

## 5.4 Runtime 切替

Runtime 中に font 切替可能な構造を考慮する。

# 6. 想定構造

```text
assets/fonts/
 ├─ default/
 │   └─ NotoSansCJK-Regular.ttc
 │
 └─ custom/
     └─ user font
```

# 7. 方針

## 7.1 Framework 標準提供

Framework 側で標準 font を提供する。

## 7.2 利用者拡張

利用者が custom font を追加可能とする。

## 7.3 OS 非依存

固定 OS path を使用しない。

## 7.4 将来拡張

以下を将来対応候補とする。

- Emoji
- Font fallback chain
- Multi language
- Font weight
- Icon font

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1b EmbeddedFont 技術検証