<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260527-009801Z-T4F2
lang: ja-JP
canonical_title: P0-1a 日本語Font 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検討目次](../技術検討目次.md) > P0-1a 日本語Font 技術検証

# P0-1a 日本語Font 技術検証

# 1. 目的

Linux 環境における egui の日本語表示成立性を確認する。

# 2. 確認項目

- 日本語表示
- Font fallback
- Noto CJK
- egui custom font
- Docking UI 日本語

# 3. 関連ドキュメント

- [技術検証仕様](01_仕様/01_日本語Font_技術検証仕様.md)
- [検証ケース](02_検証仕様/01_検証ケース.md)
- [検証結果](03_検証結果/README.md)

# 4. Ubuntu 環境

```bash
sudo apt update

sudo apt install -y fonts-noto-cjk
```

# 5. 実行

```bash
cd docs/ja-JP/90_技術検証/P0-1a_日本語Font_技術検証
cargo run
```

# 6. 成功条件

- 日本語が □ 表示にならない
- Docking UI 上で日本語表示可能
- Window resize 後も表示崩れしない

---

[目次](../../目次.md) > [技術検討目次](../技術検討目次.md) > P0-1a 日本語Font 技術検証