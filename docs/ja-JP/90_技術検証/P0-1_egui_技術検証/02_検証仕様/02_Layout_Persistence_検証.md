<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260525-000701Z-Q4N2
lang: ja-JP
canonical_title: Layout Persistence 検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > Layout Persistence 検証

# Layout Persistence 検証

# 1. 目的

Dock Layout の保存・復元を確認する。

# 2. 検証対象

- Panel 移動
- Layout 保存
- Layout 復元
- DockState serialize
- DockState deserialize

# 3. 実施手順

## 3.1 Panel 移動

Docking UI 上で Panel を移動する。

## 3.2 Layout 保存

Save Layout を実行する。

## 3.3 再起動

Application を再起動する。

## 3.4 Layout 復元

前回 Layout が復元されることを確認する。

# 4. 成功条件

- Layout が保存される
- 再起動後も Layout が維持される

---

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > Layout Persistence 検証