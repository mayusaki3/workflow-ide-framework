<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260525-000101Z-L2Q8
lang: ja-JP
canonical_title: Docker Linux Build 検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > Docker Linux Build 検証

# Docker Linux Build 検証

# 1. 目的

Linux 環境で Rust / egui / eframe / egui_dock が build 可能か確認する。

# 2. 検証対象

- cargo build
- dependency resolve
- egui compile
- eframe compile
- egui_dock compile
- serde compile

# 3. 非対象

以下は本検証では対象外。

- GUI 表示
- Docking 操作
- GPU Viewport
- WebView

# 4. 実施手順

## 4.1 Docker build

```bash
docker build -t p0-1-egui-validation .
```

## 4.2 Docker run

```bash
docker run --rm p0-1-egui-validation
```

# 5. 成功条件

- cargo build 成功
- build error が発生しない

# 6. 判定

本検証は Linux build portability 確認として扱う。

---

[目次](../../../目次.md) > [技術検討目次](../../技術検討目次.md) > [P0-1 egui 技術検証](../README.md) > Docker Linux Build 検証