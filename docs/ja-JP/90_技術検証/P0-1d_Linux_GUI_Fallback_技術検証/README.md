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

## 2.2 検証仕様

- [Linux GUI Fallback 検証ケース](./02_検証仕様/01_検証ケース.md)

# 3. 検証対象

本検証は、P0-1d 独自の Cargo project を実行するものではない。

P0-1c の Cargo project を Linux fallback 条件付きで起動し、fallback 起動方式の有効性を検証する。

検証対象 project:

```text
../P0-1c_Linux_Native_Window_Title_技術検証
```

検証対象:

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

# 5. 実行方法

## 5.1 Font setup

P0-1d では EmbeddedFont あり構成で fallback を確認する。

font は P0-1c 配下へ配置する。

### Linux

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

chmod +x ./scripts/setup_fonts.sh

./scripts/setup_fonts.sh
```

### Windows PowerShell

```powershell
cd C:\WORKPLACE\Makes\GitHub\workflow-ide-framework\docs\ja-JP\90_技術検証\P0-1c_Linux_Native_Window_Title_技術検証

./scripts/setup_fonts.ps1
```

## 5.2 P0-1c fallback script による実行

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

chmod +x ./scripts/run_linux_fallback.sh

./scripts/run_linux_fallback.sh
```

fallback script は P0-1c 配下で管理する。

理由:

- fallback の対象 project が P0-1c
- Cargo project 側で target 管理するため
- working tree 汚染を避けるため
- fallback 実行と通常実行を同一 project 配下で比較するため

## 5.3 比較用の直接実行

P0-1c を fallback なしで直接実行する場合は、以下を使用する。

```bash
cd ~/workflow-ide-framework/docs/ja-JP/90_技術検証/P0-1c_Linux_Native_Window_Title_技術検証

cargo run
```

# 6. 記録方針

P0-1d の検証結果には、以下を分けて記録する。

- P0-1c fallback なし直接実行結果
- P0-1c fallback あり実行結果
- fallback により改善した項目
- fallback でも改善しない項目
- EmbeddedFont 有無による差異

# 7. 想定方針

Linux GUI backend failure 時は fallback 起動を許容する。

ただし、fallback は native window title 日本語表示を改善するためではなく、Linux GUI backend 起動安定化のための回避策として扱う。

# 8. 今後の確認対象

- WebView
- WebKitGTK
- IME
- 日本語入力
- GPU backend
- Vulkan/OpenGL

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-1d Linux GUI Fallback 技術検証