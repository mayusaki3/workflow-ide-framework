<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009101Z-L6T1
lang: ja-JP
canonical_title: P0-1 egui 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検討目次](../技術検証目次.md) > P0-1 egui 技術検証

# P0-1 egui 技術検証

# 1. 目的

Rust + egui + eframe を利用した Runtime IDE 構造の成立性を確認する。

本検証は、以後の GPU Viewport / WebView / Runtime 分離の前提確認として扱う。

# 2. 検証項目

- Window 表示
- Docking
- Multi Panel
- GPU Viewport
- WebView coexist
- Event Loop
- State 更新

# 3. 構成

- 01_仕様
- 02_検証仕様
- 03_検証結果
- src

# 4. 実行方法

## 4.1 前提条件

- Rust toolchain 導入済み
- cargo 利用可能

## 4.2 実行コマンド

```powershell
cd docs/ja-JP/90_技術検証/P0-1_egui_技術検証
cargo run
```

# 5. 成功条件

以下を満たすこと。

- Window が表示される
- Docking UI が表示される
- Status / Viewport / Log Panel が表示される
- Panel を移動可能である
- Window が異常終了しない

# 6. 検証結果記録

検証結果は以下へ記録する。

- 03_検証結果/README.md

失敗時は以下を記録する。

- OS
- Rust version
- cargo version
- Error log
- 再現手順

# 7. 現在の状態

P0-1 は未検証。

P0-2 WebView 技術検証は、P0-1 完了後に継続評価する。

---

[目次](../../目次.md) > [技術検討目次](../技術検証目次.md) > P0-1 egui 技術検証