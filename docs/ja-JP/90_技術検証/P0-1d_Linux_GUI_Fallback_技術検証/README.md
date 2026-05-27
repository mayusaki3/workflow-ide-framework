<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260528-010201Z-G8W3
lang: ja-JP
canonical_title: P0-1d Linux GUI Fallback 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1d Linux GUI Fallback 技術検証

# P0-1d Linux GUI Fallback 技術検証

# 1. 目的

Linux GUI backend 問題に対する fallback 運用方針を検証する。

# 2. 関連ドキュメント

## 2.1 背景

- [P0-1c Linux Native Window Title 技術検証](../P0-1c_Linux_Native_Window_Title_技術検証/README.md)

# 3. 検証対象

- software renderer fallback
- X11 fallback
- Hyper-V Ubuntu Desktop
- GUI backend stability
- Runtime 起動方針

# 4. 想定 fallback

```bash
LIBGL_ALWAYS_SOFTWARE=1
WINIT_UNIX_BACKEND=x11
```

# 5. 想定方針

Linux GUI backend failure 時は fallback 起動を許容する。

# 6. 今後の確認対象

- WebView
- WebKitGTK
- IME
- 日本語入力
- GPU backend
- Vulkan/OpenGL

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1d Linux GUI Fallback 技術検証