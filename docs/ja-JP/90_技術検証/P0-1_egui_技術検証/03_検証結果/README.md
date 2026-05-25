<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009401Z-P7H8
lang: ja-JP
canonical_title: P0-1 検証結果
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > P0-1 検証結果

# P0-1 検証結果

# 1. 状態

実施中。

# 2. 実施結果

## 2.1 cargo build

依存取得成功。

- eframe
- egui
- egui_dock

の取得および build 開始を確認した。

## 2.2 API 差異

以下の API 差異を確認した。

### eframe

- run_native API 差異
- App trait API 差異

### egui_dock

- show() deprecated
- show_inside() へ変更必要

## 2.3 修正方針

以下へ追従する。

- eframe 0.34 系
- egui_dock 最新系

# 3. 現在の状態

ソース修正後、再実行待ち。

# 4. 今後の確認項目

- Window 表示
- Docking
- Multi Panel
- GPU Viewport Placeholder
- egui_dock

# 5. 想定リスク

- egui_dock version 差異
- eframe backend 差異
- OS 依存 UI 差異
- GPU backend 差異

---

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > P0-1 検証結果