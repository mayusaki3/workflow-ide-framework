<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-009901Z-U7K8
lang: ja-JP
canonical_title: 日本語Font 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1a 日本語Font 技術検証](../README.md) > 日本語Font 技術検証仕様

# 日本語Font 技術検証仕様

# 1. 概要

Linux + egui における日本語表示成立性を確認する。

# 2. 検証対象

- egui
- eframe
- Noto CJK
- Font fallback

# 3. 確認事項

## 3.1 日本語表示

日本語を正常表示可能であること。

## 3.2 Docking UI

Docking UI 上で日本語表示可能であること。

## 3.3 Font Load

custom font load を確認する。

## 3.4 Resize

Window resize 後も表示崩れしないこと。

# 4. 今後の拡張

- Runtime Log 日本語
- Workflow Node 日本語
- Multi Font
- Emoji

---

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1a 日本語Font 技術検証](../README.md) > 日本語Font 技術検証仕様