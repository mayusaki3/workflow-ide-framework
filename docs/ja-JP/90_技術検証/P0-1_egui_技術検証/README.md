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

# 2. 関連ドキュメント

- [技術検証仕様](01_仕様/01_技術検証仕様.md)
- [検証ケース](02_検証仕様/01_検証ケース.md)
- [検証結果](03_検証結果/README.md)

# 3. 検証項目

- Window 表示
- Docking
- Multi Panel
- GPU Viewport
- WebView coexist
- Event Loop
- State 更新

# 4. 構成

- 01_仕様
- 02_検証仕様
- 03_検証結果
- src

# 5. Git 取得

```bash
git clone https://github.com/mayusaki3/workflow-ide-framework.git

cd workflow-ide-framework

git checkout develop
```

# 6. Ubuntu Desktop 環境構築

## 6.1 egui / eframe

```bash
sudo apt update

sudo apt install -y \
    build-essential \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libssl-dev \
    curl \
    git
```

## 6.2 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

# 7. Docker 環境

# 7.1 build

```bash
docker build -t p0-1-egui-validation .
```

# 7.2 run

```bash
docker run --rm -it p0-1-egui-validation
```

# 7.3 cleanup

## Container / Network / Volume

```bash
docker compose down -v
```

## Image 削除

```bash
docker image rm p0-1-egui-validation
```

## 未使用 Docker 資源削除

```bash
docker system prune -a --volumes
```

# 8. 実行方法

## 8.1 前提条件

- Rust toolchain 導入済み
- cargo 利用可能

## 8.2 実行コマンド

```bash
cd docs/ja-JP/90_技術検証/P0-1_egui_技術検証
cargo run
```

# 9. 成功条件

以下を満たすこと。

- Window が表示される
- Docking UI が表示される
- Status / Viewport / Log Panel が表示される
- Panel を移動可能である
- Window が異常終了しない

# 10. 検証結果記録

検証結果は以下へ記録する。

- 03_検証結果/README.md

失敗時は以下を記録する。

- OS
- Rust version
- cargo version
- Error log
- 再現手順

# 11. 現在の状態

P0-1 は未検証。

P0-2 WebView 技術検証は、P0-1 完了後に継続評価する。

---

[目次](../../目次.md) > [技術検討目次](../技術検証目次.md) > P0-1 egui 技術検証