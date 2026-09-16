<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-11-07 Browser Surface実験用インターフェース整理
document_type: testspec
canonical_document: false
-->

# WV-11-07 Browser Surface 実験用インターフェース整理

## 目的

WV-11 の技術検証で成立した CEF OSR →描画バッファ→egui Texture→Dock の経路を、Framework 内部で扱うための最小インターフェースとして整理する。

ここで定義するものは技術検証用の実験的内部インターフェースであり、Framework 利用アプリ向け正式 API ではない。GPU Surface との共通化、他 Dock Panel との統一、公開 API の命名と互換性は WV-12 および P0-2 完了後の仕様フェーズで決定する。

## 境界

利用アプリおよび Dock 上位層へ CEF、Windows IMM、GTK、Wayland、X11 等の OS / Browser Runtime 固有型を公開しない。

Browser Runtime 固有処理は Browser Surface 実装内部へ閉じ込め、Dock 側との境界は Surface のサイズ、描画更新、入力、ライフサイクル、エラーだけとする。

## 実験用インターフェース

以下は Rust の正式 API ではなく、責務とデータフローを確認するための概念インターフェースである。

```text
BrowserSurface
  create(config) -> Result<BrowserSurface, SurfaceError>
  load_url(url) -> Result<(), SurfaceError>
  resize(width, height, scale_factor) -> Result<(), SurfaceError>
  input(event) -> Result<(), SurfaceError>
  poll_frame() -> Option<SurfaceFrame>
  set_focus(focused) -> Result<(), SurfaceError>
  close() -> Result<(), SurfaceError>

BrowserSurfaceConfig
  initial_url
  initial_width
  initial_height
  scale_factor

SurfaceFrame
  width
  height
  generation
  pixel_buffer / texture_source

SurfaceInputEvent
  PointerMove
  PointerButton
  Wheel
  Key
  Text / Composition   // 実験用。IME詳細はプラットフォーム実装内部

SurfaceError
  RuntimeLoad
  Initialize
  BrowserCreate
  Render
  Input
  Shutdown
  Platform
```

## 責務

### Browser Surface 作成

`create` は Browser Runtime の初期化と Windowless Browser 作成に必要な処理を実装内部へ閉じ込める。呼び出し側は CEF の型や Native Window Handle を要求されない。

### URL 読み込み

`load_url` は Browser Surface の表示内容変更だけを表現する。CEF の Frame / BrowserHost 等を公開 API 境界へ出さない。

### リサイズ

`resize` は Dock が確定した論理表示サイズと scale factor を Browser Surface へ通知する境界とする。Runtime 固有の `was_resized` 等は実装内部で処理する。

### 入力イベント

`input` は Pointer Move / Button / Wheel / Keyboard 等を OS 非依存イベントとして受ける。OS ネイティブイベント、CEF Event 型への変換は Browser Surface 実装内部で行う。

IME は Windows 追加検証で OS 固有処理が必要であることが確認されているため、Composition の抽象化境界だけを候補として残し、正式な共通イベント形状は本項目では確定しない。

### Texture 更新

`poll_frame` は新しい描画世代がある場合だけ SurfaceFrame を返す概念とする。WV-11 では CPU 側描画バッファから egui Texture を更新する経路を成立確認済みとし、GPU Texture の直接共有は WV-12 の検証対象とする。

### Focus

`set_focus` は Browser Surface が入力対象かどうかを明示する。Native focus や Browser Runtime の focus API は実装内部で処理する。

### ライフサイクル

`close` は Browser の終了、必要なメッセージ処理、Runtime shutdown の責務を持つ。終了順序を利用アプリ側へ要求しない。

### エラー通知

`SurfaceError` は利用側が Runtime 固有エラー型へ依存しないための分類境界とする。詳細ログには Runtime / OS 固有情報を保持してよいが、上位層の分岐条件にはしない。

## WV-11 で確定しない事項

- 公開 Rust trait / struct / enum の最終名称
- async / sync API の選択
- Browser Runtime の所有単位と複数 Surface 間共有
- GPU Texture の直接受け渡し
- GPU Surface と Browser Surface の共通 trait
- IME Composition の正式な共通イベントモデル
- Drag and Drop
- Clipboard
- Accessibility
- Browser security / sandbox policy
- Runtime 配布・更新方式

これらは WV-11 の成立性判定には含めない。

## OS 差異の扱い

Windows 実機および Linux VM / Wayland で CEF OSR → Surface 化が成立しているため、上位層を OS 非依存境界にできる見込みがある。

Linux 実機、Linux X11 実機、macOS Apple Silicon、macOS Intel は未検証であり `？` のままとする。未検証は非対応を意味しない。

## 判定

WV-11-07 の目的である「技術検証に必要な最小インターフェースを正式仕様と分離して記録する」は完了とする。

この文書を正式 API 仕様として参照してはならない。WV-12 GPU Surface 技術検証後、Surface 共通化の結果を踏まえて正式仕様を作成する。
