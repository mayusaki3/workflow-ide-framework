<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260529-010201Z-L4Q8
lang: ja-JP
canonical_title: EmbeddedFont Asset Setup
document_type: note
canonical_document: true
-->

[目次](../../../../../目次.md) > [技術検証目次](../../../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../../../README.md) > EmbeddedFont Asset Setup

# EmbeddedFont Asset Setup

# 1. 概要

本ディレクトリには P0-1c / P0-1d 検証で使用する Embedded Font asset を配置する。

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
 └─ NotoSansCJKjp-Regular.otf
```

# 4. 理由

以下理由により repository へ font asset 本体は含めない。

- ライセンス管理
- repository size
- 配布管理
- 将来 custom font 対応

# 5. 注意

P0-1c 単独検証では通常実行しない。

P0-1d fallback 検証など、EmbeddedFont 有り条件で比較する場合に使用する。

# 6. 今後の検討

- hash verification
- font cache
- multiple font package
- icon font
- emoji font

---

[目次](../../../../../目次.md) > [技術検証目次](../../../../技術検証目次.md) > [P0-1c Linux Native Window Title 技術検証](../../../README.md) > EmbeddedFont Asset Setup