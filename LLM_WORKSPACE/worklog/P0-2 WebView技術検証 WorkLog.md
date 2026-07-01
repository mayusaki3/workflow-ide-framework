目次 > LLM_WORKSPACE > worklog > P0-2 WebView技術検証 WorkLog

# P0-2 WebView技術検証 WorkLog

## 現在状態

### 到達点

- WV-10 完了。
- GTK / WebKitGTK Host Window 方式は Linux 主方式として終了する。
- WV-10 の X11 reparent 確認では、Wayland セッションのため X11 reparent 検証が成立しないことを確認した。
- WV-10 で GTK / WebKitGTK 生成なしの場合に応答なしが消えることを確認した。
- WV-10 で GTK / WebKitGTK 表示時の応答なし再現を確認した。
- WV-09 系検証完了。
- WV-08 系検証完了。

## P0-2 の残作業

P0-2 は、Windows を主対象として Surface 共通アーキテクチャの成立性を確認する方向へ移行する。

P0-2 では、仕様設計、Runtime 統合、IDE 統合までは行わない。

P0-2 の残作業は以下の 2 検証で終了する。

- WV-11 Browser Surface 技術検証
- WV-12 GPU Surface 技術検証

## 更新対象

- src/platform/linux_webview.rs
- WV-11_Browser_Surface成立性検証.md
- WV-12_GPU_Surface成立性検証.md
- WV-10_WV07再現条件差分分析.md
- WV-09_Linux応答なし原因特定.md

## 現在の判断

WV-10 までの検証により、eframe / egui / Dock のみでは応答なしは発生せず、GTK / WebKitGTK を生成・表示した場合に応答なしが発生することを確認した。

Wayland 環境では Windows 版の Child Window や X11 reparent 相当の方式を主方式にできないため、Window 埋め込み方式ではなく、Surface として描画結果を Dock 内へ統合する方式を検討する。

Windows 版は現状 WebView2 によりブラウザ表示が成立しているが、Windows / Linux の実装差を減らし、GPU Surface と共通化する観点では、Windows / Linux 共通で CEF OSR（Off-Screen Rendering）を用いた Browser Surface 方式を検証する価値が高い。

## サポート優先度

1. Windows
2. Linux
3. macOS

macOS は現時点で検証環境がないため、P0-2 ではサポート対象外とする。

ただし、Surface 共通アーキテクチャは macOS 実装を将来追加できる構造を維持する。

## 利用アプリ向け方針

Framework 利用アプリ開発者には、OS 依存コードを書かせない方針とする。

OS 差異は Framework 側の Surface API が吸収する。

利用アプリ側は、Browser Surface / GPU Surface を OS 非依存の API として利用できることを目標とする。

P0-2 完了後、Framework 利用アプリ側と API を調整し、Surface API の仕様策定と Framework 本体実装へ進む。

## WV-11 Browser Surface 技術検証

Windows を主対象として、Dock 上で Web ブラウザ機能を Browser Surface として成立させられるか確認する。

主候補は CEF OSR とする。

確認項目:

- WV-11-01: Rust から利用可能な CEF バインディング / ラッパーの現状確認
- WV-11-02: CEF OSR で描画バッファを取得できるか確認
- WV-11-03: 取得した描画バッファを egui TextureHandle へ転送できるか確認
- WV-11-04: egui 側イベントを CEF へ転送できるか確認
- WV-11-05: Windows 上で Dock 内 Browser Surface として表示できるか確認
- WV-11-06: Linux 上で同方式の成立性を確認
- WV-11-07: 利用アプリ向け Browser Surface API の仮インターフェース案を整理

## WV-12 GPU Surface 技術検証

Windows を主対象として、Dock 上で GPU サーフェース機能を Surface として成立させられるか確認する。

Browser Surface と同じ Surface 共通モデルへ統合可能かを確認する。

確認項目:

- WV-12-01: GPU Surface 実現方式の候補確認
- WV-12-02: GPU 描画結果を egui / Dock 内 Texture として表示できるか確認
- WV-12-03: Resize / Visibility / Lifecycle の基本動作確認
- WV-12-04: イベントを GPU Surface へ転送できるか確認
- WV-12-05: Browser Surface と共通化可能な Surface API 要素を整理
- WV-12-06: 利用アプリ向け GPU Surface API の仮インターフェース案を整理

## P0-2 完了条件

- WV-11 Browser Surface 技術検証が完了している。
- WV-12 GPU Surface 技術検証が完了している。
- Browser Surface と GPU Surface が、共通 Surface モデルとして成立する見込みを確認できている。
- Windows を主対象として、Dock 上で Browser Surface / GPU Surface が成立することを確認できている。
- Linux について、同方式を採用できる見込みを判断できている。
- macOS は未検証だが、将来実装を追加可能な構造であることを確認できている。
- Framework 利用アプリ側へ OS 非依存 API として提供できる見込みを確認できている。

## P0-2 完了後の移行先

P0-2 完了後は、技術検証ではなく仕様フェーズへ移行する。

仕様側で扱う内容は以下とする。

- Surface 共通 API 仕様
- Browser Surface 仕様
- GPU Surface 仕様
- Runtime 統合仕様
- Framework API 仕様
- Framework 利用アプリ向け API 仕様
- IDE 統合仕様

これらは WV-13 以降の技術検証ではなく、本来の仕様側作業として扱う。

## 次工程

WV-11 Browser Surface 技術検証を開始する。

最初の作業は WV-11-01 CEF OSR Rust 利用方式調査とする。

---

目次 > LLM_WORKSPACE > worklog > P0-2 WebView技術検証 WorkLog
