<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-12-05 Browser SurfaceとGPU Surface比較整理
document_type: note
canonical_document: false
-->

# WV-12-05 Browser Surface と GPU Surface 比較整理

## 目的

WV-11 Browser Surface と WV-12 GPU Surface の技術検証結果を比較し、P0-2 完了後に共通 Surface API を設計するための境界を整理する。

本書は正式 API 仕様ではない。

## 比較結果

| 項目 | Browser Surface | GPU Surface | 共通化判断 |
| --- | --- | --- | --- |
| Dock 内表示 | CEF OSR 描画結果を egui Texture 化 | wgpu Texture を egui に登録 | 共通化候補 |
| Frame / Texture 更新 | CPU 描画バッファ起点 | GPU Texture 起点 | 上位の更新通知は共通化候補。実データ型は分離候補 |
| Resize | Browser Runtime へサイズ通知 | GPU Texture 再生成 | 共通化候補 |
| Visibility | Browser 表示状態管理 | Surface 表示状態管理 | 共通化候補 |
| Lifecycle | Browser / Runtime の生成終了 | GPU Resource の生成破棄 | 共通化候補 |
| PointerMove | CEF Event へ変換 | GPU Surface 入力へ転送 | 共通入力イベント候補 |
| PointerButton | CEF Event へ変換 | GPU Surface 入力へ転送 | 共通入力イベント候補 |
| Wheel | CEF Event へ変換 | GPU Surface 入力へ転送 | 共通入力イベント候補 |
| Keyboard | CEF Event へ変換 | GPU Surface 入力へ転送 | 共通入力イベント候補 |
| Focus | BrowserHost 側へ反映 | Surface 入力対象状態 | 共通化候補 |
| Text / IME | Browser / OS 固有処理が必要 | 基本文字入力は転送可能 | 共通境界は候補、IME詳細は分離 |
| URL / Navigation | 必要 | 不要 | Browser 固有 |
| Browser Runtime | 必要 | 不要 | Browser 固有 |
| GPU Device / Queue / Texture | egui 表示側以外は必須でない | 描画実装で必要 | GPU 固有 |
| Render pipeline | Browser Runtime 内部 | GPU Surface 実装責務 | GPU 固有 |

## 共通化可能な責務

上位 Dock / Surface 境界では以下を共通化できる見込みがある。

```text
Surface
  resize(width, height, scale_factor)
  input(event)
  set_focus(focused)
  set_visible(visible)
  close()

SurfaceInputEvent
  PointerMove
  PointerButton
  Wheel
  Key
  Text / Composition  // 詳細形状は未確定
```

描画結果については Browser Surface と GPU Surface で生成元が異なるため、`SurfaceFrame` を単一の CPU pixel buffer 型へ固定しない。

上位層が必要とするのは「Dock 内へ表示可能な描画結果が更新されたこと」であり、CPU buffer / native GPU texture / texture source の差は実装側へ閉じ込める方向を候補とする。

## 共通化すべきでない責務

Browser Surface 固有:

- URL / Navigation
- Browser Runtime 初期化と shutdown
- CEF Browser / Frame / BrowserHost
- Browser security / sandbox
- Browser 固有 IME bridge

GPU Surface 固有:

- GPU Device / Queue
- RenderPass / RenderPipeline
- GPU Texture 作成と再生成
- GPU resource synchronization
- Backend 固有最適化

これらを共通 Surface trait へ持ち込まない。

## IME の扱い

WV-11 では Windows IME に OS / Browser Runtime 固有 bridge が必要だった。一方 WV-12-04 では基本的な Key / Text 入力転送は Surface 境界で成立した。

したがって `Text / Composition` を上位入力モデルの候補として残すが、IME candidate window、composition range、IMM / Browser Runtime API 等は共通 Surface の責務にしない。

## 描画境界の判断

Browser Surface は CEF OSR の CPU 描画バッファ、GPU Surface は wgpu の GPU Texture を起点とするため、共通 API を `pixel_buffer` に固定すると GPU Surface に不要な readback / copy を要求する可能性がある。

逆に GPU Texture だけへ固定すると Browser Surface の成立済み CPU buffer 経路へ不要な GPU 固有依存を持ち込む。

このため、正式仕様では Surface の描画成果物を抽象化し、実装ごとの Texture source / frame source を内部 adapter で Dock 描画へ接続する構造を候補とする。

zero-copy や GPU 間 Texture 共有は性能最適化として別途扱い、P0-2 の成立条件にはしない。

## 判定

WV-12-05 の目的である、Browser Surface と GPU Surface の共通化可能な要素と固有要素の整理は完了とする。

共通化候補は Resize、Visibility、Lifecycle、基本入力、Focus、描画更新通知である。

Browser Runtime / Navigation / Browser IME bridge と、GPU Device / Render pipeline / GPU resource 管理は各 Surface 実装固有とする方向が妥当である。

正式 trait、struct、enum、async / sync、描画成果物の具体型は本項目では確定しない。

次は WV-12-06 で GPU Surface の実験用内部インターフェースを整理する。
