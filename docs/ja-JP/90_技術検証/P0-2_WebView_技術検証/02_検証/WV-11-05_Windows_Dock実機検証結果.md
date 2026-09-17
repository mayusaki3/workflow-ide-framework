# WV-11-05 Windows Dock 実機検証結果

## 位置付け

本記録は `WV-11_Browser_Surface成立性検証.md` の WV-11-05 Windows Dock 表示検証に対する Windows 実機結果である。

Browser Surface は Dock Panel 内のコンテンツとして扱う。1 タブ = 1 区画という UI 構造は前提としないため、Dock 移動およびタブ切替は WV-11-05 の Browser Surface 成立性の判定対象外とする。

## 検証環境

- OS: Windows 実機
- Branch: `develop`
- Runner: `run_wv11_05_windows_dock_probes.ps1`

## 検証結果

| 動作対象 | Dock 内表示 | リサイズ追従 | 基本入力 |
| --- | --- | --- | --- |
| Windows 実機 | ○ | ○ | ○ |

### Dock 内表示

`cef_dock_probe` を実行し、CEF OSR Paint が `800x600` で受信され、egui Texture が生成されて Browser Surface が Dock Panel 内へ表示されることを確認した。

### リサイズ追従

`cef_dock_resize_probe` を実行し、アプリケーションウィンドウの連続リサイズに応じて CEF OSR resize request と Paint 更新が継続することを確認した。

初期 `800x600` から `972x646`、`970x643`、`956x624` など複数サイズへの更新が観測され、リサイズ中も Browser Surface を失わず追従した。

### 基本入力

`cef_input_keyboard_probe` を実行し、Browser Surface を Focus 後、以下の keydown / keyup が CEF Browser 側へ転送されることを確認した。

- A
- Enter
- ArrowUp
- ArrowLeft
- ArrowDown
- ArrowRight

## 判定

Windows 実機について、WV-11-05 で Browser Surface に要求する Dock Panel 内表示、リサイズ追従、基本入力は成立したため `○` とする。

Dock 移動およびタブ挙動は未確認ではなく、本 Framework の Browser Surface 成立性では 1 タブ = 1 区画を前提としないため、本項目の判定対象外とする。

OS 非依存 API としての整理は WV-11-07 の実験用インターフェース整理で扱う。
