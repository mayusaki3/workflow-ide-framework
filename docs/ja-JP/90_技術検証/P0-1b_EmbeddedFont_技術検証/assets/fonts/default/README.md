<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-010501Z-A3N7
lang: ja-JP
canonical_title: EmbeddedFont Asset Setup
document_type: note
canonical_document: true
-->

[目次](../../../../../目次.md) > [技術検証目次](../../../../技術検証目次.md) > [P0-1b EmbeddedFont 技術検証](../../../README.md) > EmbeddedFont Asset Setup

# EmbeddedFont Asset Setup

# 1. 概要

本ディレクトリには Embedded Font 技術検証用 font asset を配置する。

font asset 本体は repository へ含めない。

# 2. setup script

## Windows

```powershell
scripts/setup_fonts.ps1
```

## Linux

```bash
./scripts/setup_fonts.sh
```

# 3. 配置結果

```text
assets/fonts/default/
 └─ NotoSansCJK-Regular.ttc
```

# 4. 理由

以下理由により repository へ font asset 本体は含めない。

- ライセンス管理
- repository size
- 配布管理
- 将来 custom font 対応

# 5. 今後の検討

- hash verification
- font cache
- multiple font package
- icon font
- emoji font

---

[目次](../../../../../目次.md) > [技術検証目次](../../../../技術検証目次.md) > [P0-1b EmbeddedFont 技術検証](../../../README.md) > EmbeddedFont Asset Setup