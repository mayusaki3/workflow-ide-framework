<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: Surface アーキテクチャー検討
document_type: note
canonical_document: false
-->

# Surface アーキテクチャー検討

## 1. 目的

P0-2 WebView 技術検証の WV-11 Browser Surface / WV-12 GPU Surface の成立結果を、既存の正式仕様へ反映する前に整理する。

本書は検討資料であり、正式 Surface API 仕様ではない。

## 2. 既存仕様との不整合

P0-2 完了時点で、既存仕様には次の旧前提が残っている。

- 全体アーキテクチャーは WebView に `wry` を採用すると記載している。
- WebView アーキテクチャーも `wry` を Panel 単位で扱う構造としている。
- GPU Viewport は独立した `Viewport trait` を前提としている。
- Browser / GPU の描画・入力・Resize・Focus・Lifecycle を横断する共通 Surface 境界が正式仕様に存在しない。

P0-2 では Native Child Window / GTK Host Window を共通方式の前提とせず、Browser Surface は CEF OSR、GPU Surface は wgpu により Dock 内描画が成立した。

したがって、既存仕様をそのまま拡張するのではなく、Surface を UI / Dock と Browser / GPU 実装の中間境界として追加する必要がある。

## 3. 配置方針

Surface は WebView 専用機能でも GPU Viewport 専用機能でもないため、`06_GPU_Viewport` または `07_WebView` の配下だけに置かない。

正式仕様では `02_仕様` 直下の共通アーキテクチャーとして Surface 境界を定義し、GPU Viewport / WebView がそれを利用する構造を候補とする。

概念構造:

```text
Dock / Panel
    |
    v
Surface boundary
    |----------------------|
    v                      v
Browser Surface        GPU Surface
    |                      |
 CEF OSR                  wgpu
```

## 4. 共通 Surface 責務候補

P0-2 の成立結果から、共通化候補は以下とする。

- Resize
- Visibility
- Lifecycle
- PointerMove
- PointerButton
- Wheel
- Keyboard
- Text / Composition の上位境界
- IME Composition（Browser / GPU 共通入力能力）
- caret / composition target 位置通知
- Focus
- 描画更新通知
- 描画成果物を Dock へ接続する抽象境界
- 共通エラー分類

描画成果物は CPU pixel buffer または GPU Texture の一方へ固定しない。

Browser Surface の CEF OSR は CPU 描画バッファを起点とし、GPU Surface は wgpu Texture を起点とするためである。

## 5. Surface 固有責務

Browser Surface 固有:

- URL / Navigation
- Browser Runtime
- CEF Browser / Frame / BrowserHost
- Browser security / sandbox
- 共通 IME event を CEF / OS API へ変換する Browser Runtime 固有 IME bridge

GPU Surface 固有:

- GPU Device / Queue
- RenderPass / RenderPipeline
- GPU Texture の生成 / 再生成
- GPU resource synchronization
- Backend 固有処理
- GPU Viewport / Runtime が文字入力を必要とする場合の共通 IME event の転送

GPU Surface 自体に文字入力を必須とはしないが、GPU Surface 上の独自 UI や 3D UI が文字入力を持つ場合は共通 Surface IME model を利用する。Candidate Window の位置指定は Surface から caret / composition target rectangle を返し、Framework が OS IME へ接続する方式を候補とする。

これらを共通 Surface trait に直接持ち込まない。

## 6. 既存 GPU Viewport 仕様への反映方針

既存の `Viewport trait` を直ちに削除または Surface trait と同一視しない。

GPU Viewport は MuJoCo / VRM Preview / Camera / Simulation / Sensor View 等の用途固有 API を持つ可能性がある。一方 Surface は Dock との描画・入力・Lifecycle 境界である。

したがって正式仕様化では、

```text
GPU Viewport
    |
    v
GPU Surface
    |
    v
Surface boundary -> Dock
```

のように Viewport と Surface を別責務として扱えるかを確認する。

## 7. 既存 WebView 仕様への反映方針

`wry` 固定は P0-2 の最終結果と一致しないため、正式仕様更新時に見直す。

Browser Surface の成立方式は CEF OSR とし、WebView / Browser Panel は Browser Surface を利用して Dock に描画する構造を候補とする。

Command Bridge、Permission、Sandbox 等は Browser Panel / Browser Surface 固有責務として Surface 共通境界から分離する。

## 8. 正式仕様化の順序

1. 共通 Surface の責務境界を正式仕様として定義する。
2. GPU Viewport 仕様を Surface 利用構造へ更新する。
3. WebView 仕様を CEF OSR / Browser Surface 利用構造へ更新する。
4. 全体アーキテクチャーの `wry` 固定記述と技術構成表を更新する。
5. Runtime 所有、async / sync、具体的 Rust trait / struct / enum を詳細化する。
6. 後続実装で Browser / GPU の共通 adapter を実装する。

## 9. 正式仕様化前に確定しない事項

- Rust trait / struct / enum の最終名称
- async / sync
- Runtime 所有単位
- GPU Device / Queue の共有単位
- SurfaceOutput の具体型
- egui Texture ID を境界へ公開するか
- render scheduling
- zero-copy / GPU 間 Texture 共有
- Device lost / GPU reset 復旧
- IME Composition の共通イベント詳細
- caret / composition target rectangle API
- GPU Surface / GPU Viewport IME の実動作検証
- Drag and Drop / Clipboard / Accessibility

これらは技術成立性ではなく API / Runtime 設計事項として扱う。

## 10. 判定

P0-2 の結果は `07_WebView` のみへ反映するのではなく、GPU Viewport と WebView の双方から利用する共通 Surface アーキテクチャーとして正式仕様へ昇格させるのが整合的である。

次工程では、この検討を基に共通 Surface の正式仕様を作成し、その後 GPU Viewport / WebView / 全体アーキテクチャーを順に更新する。
