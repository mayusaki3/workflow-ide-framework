目次 > LLM_WORKSPACE > worklog > P0-2 WebView技術検証 WorkLog

# P0-2 WebView技術検証 WorkLog

## 現在状態

### 到達点

- WV-11 方針更新完了
- WV-10 の X11 reparent 確認では、Wayland セッションのため X11 reparent 検証が成立しないことを確認
- WV-10 で GTK / WebKitGTK 生成なしの場合に応答なしが消えることを確認
- WV-10 で GTK / WebKitGTK 表示時の応答なし再現を確認
- WV-09 系検証完了
- WV-08 系検証完了

## 更新対象

- src/platform/linux_webview.rs
- WV-11_CEF_OSR方式成立性検証.md
- WV-10_WV07再現条件差分分析.md
- WV-09_Linux応答なし原因特定.md

## 除外済み要因

- Visibility同期フェーズの GTKイベントポンプ
- WebView set_visible 継続実行
- Native Surface位置同期単独
- WebView set_bounds
- GtkFixed の通常入力イベント
- eframe / egui / Dock 単独処理

## 現在の判断

WV-10 までの検証により、eframe / egui / Dock のみでは応答なしは発生せず、GTK / WebKitGTK を生成・表示した場合に応答なしが発生することを確認した。

WV-10 では X11 前提で GTK Window を eframe / winit Window へ reparent する検証を試みたが、実行環境は Wayland セッションであり、eframe / winit も GTK も Wayland 側で動作していた。そのため X11 reparent 検証は成立しなかった。

Wayland 環境では Windows 版の Child Window や X11 reparent 相当の方式を主方式にできないため、Window 埋め込みではない方式を検討する。

次候補として、CEF OSR（Off-Screen Rendering）方式を WV-11 として検討する。CEF OSR では WebView を独立 Window や GTK Widget として配置するのではなく、ブラウザ描画結果をバッファとして取得し、egui の Texture として Dock 内へ描画する方式を想定する。

## WV-11 方針

### 目的

Wayland 環境で、eframe / egui Dock 内に WebView 相当表示を実現できるか確認する。

### 検証方針

- GTK Window 同期方式は主候補から外す。
- X11 reparent 方式は Wayland では成立しないため、Wayland 対応の主候補から外す。
- CEF OSR によるオフスクリーン描画を検証候補にする。

### 確認項目

- WV-11-01: Rust から利用可能な CEF バインディング / ラッパーの現状確認
- WV-11-02: CEF OSR で描画バッファを取得できるか確認
- WV-11-03: 取得した描画バッファを egui TextureHandle へ転送できるか確認
- WV-11-04: egui 側のマウス / キーボード入力を CEF へ転送できるか確認
- WV-11-05: Wayland セッション上で GTK Window なしに動作できるか確認

## 次工程

WV-11-01 CEF OSR Rust 利用方式調査

---

目次 > LLM_WORKSPACE > worklog > P0-2 WebView技術検証 WorkLog
