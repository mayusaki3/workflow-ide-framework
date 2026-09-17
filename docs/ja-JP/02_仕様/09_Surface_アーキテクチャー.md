<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: Surface アーキテクチャー
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > [仕様目次](./仕様目次.md) > Surface アーキテクチャー

# Surface アーキテクチャー

## 1. 概要

Surface は Dock / Panel と、Browser・GPU 等の描画実装との間に置く共通境界である。

Surface は Native Child Window 埋め込みを前提とせず、描画成果物、表示領域、入力、Focus、Visibility、Lifecycle を Framework 内部で接続可能な構造とする。

P0-2 技術検証では Browser Surface を CEF OSR、GPU Surface を wgpu で成立確認した。

## 2. 責務境界

概念構造は以下とする。

```text
Dock / Panel
    |
Surface
   / \
Browser GPU
Surface Surface
   |      |
CEF OSR  wgpu
```

Surface 共通層は Browser Runtime や GPU Backend の固有型をアプリケーション / Dock 上位層へ要求しない。

## 3. 共通責務

Surface 共通境界は以下を扱う。

- Resize
- Visibility
- Lifecycle
- PointerMove
- PointerButton
- Wheel
- Keyboard
- Text / Composition の上位入力境界
- Focus
- 描画更新通知
- 描画成果物を Dock / Panel へ接続する抽象境界
- 共通エラー境界

描画成果物の共通表現を CPU pixel buffer または GPU Texture の一方へ固定しない。

## 4. Browser Surface

Browser Surface は WebView / Browser Panel の描画境界である。

主実装方式は CEF OSR とする。

Browser Surface 固有責務は以下を含む。

- URL / Navigation
- Browser Runtime
- CEF Browser / Frame / BrowserHost
- Browser security / sandbox
- Browser Runtime / OS 固有 IME bridge

CEF OSR の CPU 描画バッファは Framework 内部で Dock / Panel が表示可能な Texture へ変換する。

## 5. GPU Surface

GPU Surface は GPU Viewport と Dock / Panel の間の描画境界である。

主実装方式は wgpu とする。

GPU Surface 固有責務は以下を含む。

- GPU Device / Queue
- RenderPass / RenderPipeline
- GPU Texture の生成 / 再生成
- GPU resource synchronization
- Backend 固有処理

GPU Device / Queue / Texture / RenderPass 等の wgpu 固有型を Surface 共通 API の必須型としない。

## 6. GPU Viewport との関係

GPU Viewport と Surface は同一責務としない。

```text
GPU Viewport
    |
GPU Surface
    |
Surface
    |
Dock / Panel
```

GPU Viewport は MuJoCo、VRM Preview、Camera、Simulation、Sensor View 等の用途固有 API を持つことができる。

Surface は用途に依存せず Dock / Panel との描画・入力・Lifecycle 境界を担当する。

## 7. 入力

共通入力境界では PointerMove / PointerButton / Wheel / Keyboard / Text / Composition / Focus を扱える構造とする。

IME は Browser Surface 固有機能とはせず、Surface 共通入力能力として扱う。

共通入力モデルは少なくとも以下の意味を表現可能とする。

- TextInput
- CompositionStart
- CompositionUpdate
- CompositionCommit
- CompositionEnd

Browser Surface では、共通 IME / Composition 入力を CEF Composition API 等へ変換する Browser 固有 bridge を持つ。

GPU Surface でも、GPU Viewport 内の独自 UI、3D 空間内 UI、ノード名編集、Simulation / Runtime 側文字入力等が文字入力を要求する場合、同じ共通 IME / Composition 入力を利用できる構造とする。純粋な GPU 描画のみを行う Surface に IME 対応を強制しない。

日本語 IME の Candidate Window 等を入力対象位置へ配置するため、文字入力を行う Surface は caret / composition target の矩形を Framework へ通知可能な構造とする。

Browser Surface の CEF Candidate Window 位置同期は、この共通位置情報を CEF / OS 固有 API へ接続する Browser 固有処理として扱う。GPU Surface 内で文字入力を実装する場合も同じ位置情報を OS IME へ接続できる構造とする。

IME Composition の最終的な共通イベント型および caret rectangle API は詳細仕様で定義する。

## 8. Lifecycle

Surface は少なくとも以下の状態変化を表現可能な構造とする。

- Create
- Resize
- Show / Hide
- Focus / Unfocus
- Update
- Close / Destroy

具体的な Rust API、同期 / 非同期方式、Runtime 所有単位は詳細仕様で定義する。

## 9. OS 差異

OS / Window System / Browser Runtime / GPU Backend の差異は可能な限り Surface 実装側で吸収する。

Framework 利用アプリ側へ Windows / Linux / macOS 固有処理を要求しない構造とする。

未検証 OS / Backend は非対応と同義に扱わない。

## 10. 日本語・Font・IME

Surface 上位 UI の日本語表示は Embedded Font を利用可能な renderer 構造を前提とする。

OS font fallback のみに依存しない。

Native Window Title は renderer 内 Font と別系統である。Linux で Native Title の日本語表示が成立しない環境に対しては、renderer 側の Custom Title Bar を利用可能な構造とする。

日本語入力の IME Composition 自体は Surface 共通入力モデルとして扱う。Browser Surface では CEF への変換、GPU Surface では GPU Viewport / Runtime 内の文字入力先への転送を各 Surface 固有 adapter が担当する。

Candidate Window 位置同期は、共通の caret / composition target 位置情報を起点とし、OS / Runtime 固有処理へ変換する。

P0-2 で確認された Windows Browser Surface の「最初の Space で Candidate Window が一度閉じ、その後再表示される」挙動は解消済みとはせず、日本語 IME の残課題として継続管理する。

## 11. 未確定事項

以下は本アーキテクチャー仕様では確定しない。

- Rust trait / struct / enum の最終名称
- async / sync
- Runtime 所有単位
- GPU Device / Queue の共有単位
- Surface output の具体型
- egui Texture ID の公開可否
- render scheduling
- zero-copy / GPU 間 Texture 共有
- Device lost / GPU reset 復旧
- IME Composition 共通イベント詳細
- caret / composition target rectangle API
- GPU Surface / GPU Viewport での IME 実動作検証
- Drag and Drop / Clipboard / Accessibility

## 12. 技術検証との対応

本仕様は P0-2 WebView 技術検証の WV-11 Browser Surface成立性検証および WV-12 GPU Surface成立性検証を基礎とする。

Windows 実機および Linux VM で成立した範囲を仕様化し、Linux 実機・macOS 等の未検証項目は対応可否を確定しない。

---

[目次](../目次.md) > [仕様目次](./仕様目次.md) > Surface アーキテクチャー
