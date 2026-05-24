<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-001601Z-G1P6
lang: ja-JP
canonical_title: GPU Viewport アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > GPU Viewport アーキテクチャー

# GPU Viewport アーキテクチャー

# 1. 概要

Workflow IDE Framework は GPU Viewport を IDE Core 構造として扱う。

# 2. GPU Viewport

GPU Viewport は Docking UI と統合する。  
複数 Viewport を扱える構造を考慮する。

# 3. Viewport API

Viewport trait により拡張可能な構造とする。

# 4. 想定用途

- MuJoCo
- VRM Preview
- Camera
- Simulation
- Sensor View

# 5. 今後の詳細仕様

- Render Lifecycle
- GPU Context
- Multi Viewport
- Render Loop
- Docking Integration

---

[目次](../../目次.md) > [仕様目次](../仕様目次.md) > GPU Viewport アーキテクチャー