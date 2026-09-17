<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-12-06 GPU Surface実験用インターフェース整理
document_type: testspec
canonical_document: false
-->

# WV-12-06 GPU Surface 実験用インターフェース整理

## 目的

WV-12 の技術検証で成立した wgpu → GPU Texture → egui → Dock の経路を、Framework 内部で扱うための最小インターフェースとして整理する。

ここで定義するものは技術検証用の実験的内部インターフェースであり、Framework 利用アプリ向け正式 API ではない。Browser Surface との共通 trait、他 Dock Panel との統一、公開 API の命名と互換性は P0-2 完了後の仕様フェーズで決定する。

## 境界

利用アプリおよび Dock 上位層へ wgpu Device / Queue / Texture / RenderPass / Backend 固有型を直接公開しない。

GPU 固有処理は GPU Surface 実装内部へ閉じ込め、Dock 側との境界は Surface のサイズ、表示状態、描画更新、入力、Focus、ライフサイクル、エラーを中心とする。

WV-12-05 の比較結果に従い、共通描画境界を CPU pixel buffer または GPU Texture のどちらか一方へ固定しない。

## 実験用インターフェース

以下は Rust の正式 API ではなく、責務とデータフローを確認するための概念インターフェースである。

```text
GpuSurface
  create(config) -> Result<GpuSurface, SurfaceError>
  resize(width, height, scale_factor) -> Result<(), SurfaceError>
  set_visible(visible) -> Result<(), SurfaceError>
  input(event) -> Result<(), SurfaceError>
  set_focus(focused) -> Result<(), SurfaceError>
  render() -> Result<(), SurfaceError>
  poll_output() -> Option<SurfaceOutput>
  close() -> Result<(), SurfaceError>

GpuSurfaceConfig
  initial_width
  initial_height
  scale_factor

SurfaceOutput
  width
  height
  generation
  source

SurfaceInputEvent
  PointerMove
  PointerButton
  Wheel
  Key
  Text / Composition   // 実験用。IME詳細は実装外またはプラットフォーム層

SurfaceError
  Initialize
  SurfaceCreate
  Resize
  Render
  Input
  Resource
  Shutdown
  Platform
```

`SurfaceOutput.source` は CPU pixel buffer、GPU texture source 等の具体型をここでは確定しない。Dock 描画へ接続可能な描画成果物を表す抽象境界とする。

## 責務

### GPU Surface 作成

`create` は GPU Surface が必要とする描画資源を生成する。wgpu Device / Queue 等の所有方式は正式仕様では未確定とし、呼び出し側へ wgpu 固有型を要求しない境界を候補とする。

WV-12-02 では wgpu off-screen Texture を生成し、egui native Texture として Dock 内へ表示できることを確認した。

### リサイズ

`resize` は Dock が確定した論理表示サイズと scale factor を GPU Surface へ通知する境界とする。

WV-12-03 では Dock サイズ変更時に旧 Texture を破棄し、新サイズの Texture を再生成できることを確認した。具体的な Texture 再生成処理は GPU Surface 実装内部へ閉じ込める。

### Visibility

`set_visible` は Surface の表示状態を明示する境界とする。

非表示時に描画を停止するか、GPU resource を維持するか、解放するかは性能・復帰時間・メモリ使用量に関わるため、本項目では固定しない。

### 入力イベント

`input` は Pointer Move / Button / Wheel / Keyboard / Text 等を Surface 入力として受ける。

WV-12-04 では PointerMove、PointerButton、Wheel、Keyboard / Text、Focus の転送経路が成立した。

OS ネイティブイベントや egui Event の具体型を正式 Surface 境界へそのまま公開しない方向を候補とする。

### Focus

`set_focus` は GPU Surface が入力対象かどうかを明示する。

WV-12-04 では Focus の取得・解除・再取得を Surface 側状態として管理できることを確認した。

### 描画

`render` は GPU Surface の描画更新を要求する概念境界とする。

実際の RenderPass / RenderPipeline / CommandEncoder / Queue submit 等は GPU Surface 実装固有とし、共通 Surface 境界へ公開しない。

常時描画、dirty 時のみ描画、外部スケジューラ駆動等の最終方式は本項目では確定しない。

### 描画成果物

`poll_output` は新しい描画世代または Dock 描画へ反映すべき成果物がある場合に `SurfaceOutput` を返す概念とする。

WV-12-02 では GPU Texture を egui native Texture として直接登録する経路を使用した。この成立結果から、GPU Surface に CPU readback を必須とする必要はない。

Browser Surface は CPU 描画バッファを起点としているため、共通 `SurfaceOutput` の具体型は正式仕様フェーズで adapter 構造と合わせて決定する。

### ライフサイクル

`close` は GPU Surface が所有する描画資源の解放を表す。

WV-12-03 では Surface の明示的 Destroy、Create / Recreate、終了時解放を制御でき、再生成後も描画を継続できることを確認した。

GPU Device / Queue を Surface 単位で所有するか Runtime 単位で共有するかは本項目では確定しない。

### エラー通知

`SurfaceError` は上位層が wgpu Backend や OS 固有エラー型へ直接依存しないための分類境界とする。

詳細ログには Adapter / Backend / Driver 等の情報を保持してよいが、上位 Surface API の型として固定しない。

## Browser Surface との共通化候補

WV-12-05 の比較結果から、以下は Browser Surface / GPU Surface の共通境界候補とする。

```text
Surface
  resize(width, height, scale_factor)
  set_visible(visible)
  input(event)
  set_focus(focused)
  close()

SurfaceInputEvent
  PointerMove
  PointerButton
  Wheel
  Key
  Text / Composition

SurfaceOutput
  width
  height
  generation
  source
```

ただし、これは共通 trait の正式定義ではない。

Browser Surface の `load_url`、Browser Runtime、CEF / IME bridge と、GPU Surface の `render`、GPU Device / Queue / RenderPipeline / resource synchronization は共通 Surface trait に含めない方向を候補とする。

## WV-12 で確定しない事項

- 公開 Rust trait / struct / enum の最終名称
- async / sync API の選択
- GPU Device / Queue の所有単位と複数 Surface 間共有
- `SurfaceOutput.source` の正式な型
- egui Texture ID を共通境界へ公開するか
- Browser Surface / GPU Surface 共通 trait の正式形状
- render scheduling
- dirty / frame request モデル
- zero-copy / GPU 間 Texture 共有
- GPU resource synchronization の正式モデル
- Backend 選択ポリシー
- Device lost / GPU reset 時の復旧モデル
- IME Composition の正式な共通イベントモデル
- Drag and Drop
- Clipboard
- Accessibility

これらは WV-12 の成立性判定には含めない。

## OS 差異の扱い

Windows 実機では wgpu GPU Surface の Dock 表示、Resize / Visibility / Lifecycle、基本入力が成立した。

Linux VM、Linux 実機、macOS は未検証項目を `？` とし、未検証を非対応とはみなさない。

wgpu を採用することで Windows / Linux の双方を同一 Rust 描画モデルで扱える設計候補は得られているが、Linux VM での実動作確認は別途行う。

## 判定

WV-12-06 の目的である「GPU Surface の技術検証に必要な最小インターフェースを正式仕様と分離して記録する」は完了とする。

この文書を正式 API 仕様として参照してはならない。

WV-11 Browser Surface と WV-12 GPU Surface の結果から、Resize、Visibility、Lifecycle、基本入力、Focus、描画更新通知を共通 Surface 境界として扱える技術的見込みが得られた。

正式 Surface API は、P0-2 完了後の仕様フェーズで他 Dock Panel との整合を含めて定義する。
