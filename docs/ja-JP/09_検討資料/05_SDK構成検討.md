[目次](../目次.md) > [検討資料目次](検討資料目次.md) > SDK 構成検討

# SDK 構成検討

## 概要

workflow-ide-framework は IDE Platform SDK として利用する。

## 開発者区分

### framework 開発者

framework 自体を開発する。

### IDE Product 開発者

framework SDK を利用して IDE を構築する。

例：

- Runtime IDE
- SansaVRM Studio AI

### 最終利用者

IDE Product を利用する。

## 想定構成

```text
workflow-ide-framework
  ↓
IDE Product
  ├─ Runtime
  ├─ Domain Model
  └─ Application Pages
```

## 将来検討

- SDK Manager
- Runtime Version
- Workspace Version
- Plugin Version

---

[目次](../目次.md) > [検討資料目次](検討資料目次.md) > SDK 構成検討