<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260628-000012Z-WV12
lang: ja-JP
canonical_title: WV-12 GPU Surface成立性検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-12 GPU Surface成立性検証

# WV-12 GPU Surface成立性検証

## 目的

Windows を主対象として、Dock 上で GPU Surface を成立させられるか確認する。

Browser Surface と GPU Surface を共通 Surface アーキテクチャとして扱える見込みを確認する。

## 背景

P0-2 の目的は、Browser Surface 単体の実装ではなく、Dock 上で Browser Surface / GPU Surface を扱える Surface 共通アーキテクチャの技術成立性を確認することである。

WV-11 では CEF OSR -> egui Texture -> Dock の経路により Browser Surface の技術成立性を確認した。

WV-12 では GPU Surface の技術成立性を確認し、Browser Surface 側に寄りすぎた実験用インターフェースになっていないかを確認する。

## 前提

- 対象ブランチは `develop` とする。
- Windows を最優先対象とする。
- Linux は次優先対象とし、利用可能な VM で基本検証を行う。
- Linux 実機および macOS を含め、検証環境を用意できない項目は `？` とし、非対応とはみなさない。
- Dock は1タブ1区画を前提としない。
- P0-2 では Surface API の本仕様化、Runtime 統合、IDE 統合までは行わない。
- WV-12 内で整理するインターフェースは技術検証用であり、正式 API 仕様ではない。
- 正式 API 仕様は、WV-11 Browser Surface 技術検証、WV-12 GPU Surface 技術検証、および他 Dock Panel との整合を踏まえ、P0-2 完了後の仕様フェーズで定義する。

## 検証状態の表記

- `○`: 検証済み・動作
- `✕`: 検証済み・非動作
- `？`: 未検証

## 検証方針

GPU Surface は、Native Window 埋め込みではなく、GPU 描画結果を Dock 内へ描画できる Surface として検証する。

評価は以下の順で行う。

1. GPU Surface 実現方式の候補確認
2. GPU 描画結果の Dock 内表示検証
3. Resize / Visibility / Lifecycle 検証
4. 入力イベント転送検証
5. Browser Surface との比較整理
6. GPU Surface 実験用インターフェース整理

## 検証項目

| 検証番号 | 項目 | 期待結果 |
| --- | --- | --- |
| WV-12-01 | GPU Surface 実現方式の候補確認 | Windows 主対象で成立しやすく、Linux へ展開可能な候補を判断できる |
| WV-12-02 | GPU 描画結果の Dock 内表示検証 | GPU 描画結果を egui / Dock 内へ表示できる |
| WV-12-03 | Resize / Visibility / Lifecycle 検証 | Dock サイズ変更、表示状態、生成破棄に追従できる |
| WV-12-04 | 入力イベント転送検証 | 入力イベントを GPU Surface 側へ転送できる |
| WV-12-05 | Browser Surface との比較整理 | Browser Surface と共通化可能な要素、共通化すべきでない要素を整理できる |
| WV-12-06 | GPU Surface 実験用インターフェース整理 | 技術検証に必要な最小インターフェースを正式仕様と分離して記録できる |

## WV-12-01 GPU Surface 実現方式の候補確認

### 目的

GPU Surface の実現方式を選定する。

Windows を主対象としつつ、Linux へ展開可能な方式を優先する。

### 評価条件

必須条件は以下とする。

- Windows 対応
- 将来 Linux 対応可能
- Dock 内表示へ統合可能
- Resize に追従可能
- 入力イベント転送可能
- Browser Surface と比較可能な Surface モデルへ整理可能

### 評価対象

| 方式 | Windows | Linux | 評価 |
| --- | --- | --- | --- |
| wgpu | 可 | 可 | 主候補 |
| DirectX 専用 | 可 | 不可 | Windows 専用のため共通方式から除外 |
| OpenGL | 可 | 可 | 互換候補 |
| Vulkan | 可 | 可 | 直接利用は検証負荷と API 固有依存が大きい |

### 選定結果

WV-12 の主検証方式は `wgpu` とする。

理由は以下とする。

- Windows / Linux の双方で同じ Rust 側の描画モデルを使用できる。
- 特定 OS の Native Child Window 埋め込みを Surface モデルの前提にする必要がない。
- DirectX / Vulkan 等の Backend 差異を上位 Surface 境界から分離できる。
- GPU 側で生成した動的な描画結果を Texture として Dock 描画へ接続する検証を構成できる。
- Browser Surface と同じく、上位層では Resize / Input / Frame または Texture 更新 / Lifecycle を中心に比較できる。

初期検証では zero-copy や GPU 間 Texture 共有を必須条件としない。まず GPU 描画 -> Dock 表示という Surface 境界の成立を優先する。

DirectX 専用方式は Windows 専用となるため共通方式から除外する。OpenGL は互換候補として残す。Vulkan 直接利用は性能面の候補にはなるが、P0-2 の共通 Surface 成立性確認としては API 固有処理が増えるため主方式にはしない。

### 判定

WV-12-01 は完了とする。

WV-12-02 では `wgpu` により GPU 描画結果を生成し、Native Child Window を使用せず egui / Dock 内へ表示する最小 Probe を作成して検証する。

## WV-12-02 GPU 描画結果の Dock 内表示検証

### 目的

GPU 描画結果を egui / Dock 内へ表示できることを確認する。

### 合格条件

- GPU 描画を行える。
- 描画結果を Dock 内へ表示できる。
- Native Window 埋め込みに依存しない。
- 継続更新できる。

### Windows 実機検証結果

| 動作対象 | GPU 描画 | Dock 内表示 | Native Child Window 不使用 | 継続更新 |
| --- | --- | --- | --- | --- |
| Windows 実機 | ○ | ○ | ○ | ○ |
| Linux VM | ○ | ○ | ○ | ○ |
| Linux 実機 | ？ | ？ | ？ | ？ |
| macOS | ？ | ？ | ？ | ？ |

`wgpu_dock_surface_probe` を Windows 実機で実行した。

実行環境では NVIDIA GeForce RTX 4050 Laptop GPU が選択され、wgpu Backend は Vulkan となった。

以下を確認した。

- wgpu off-screen Texture を GPU RenderPass で継続更新できる。
- off-screen Texture を egui native Texture として登録し、egui_dock の Dock 内へ表示できる。
- GPU Surface の色が継続変化し、GPU frame generation が増加し続ける。
- Probe Info と GPU Surface の表示を切り替えても更新が継続する。
- Application Window をリサイズしても Dock 内表示が維持される。
- Native Child Window を使用しない。

実行ログでは GPU frame generation が 4560 まで継続し、異常終了は確認されなかった。

以上により Windows 実機の WV-12-02 合格条件をすべて満たしたため `○` とする。

Linux VM では `llvmpipe (LLVM 21.1.8)` / Vulkan Backend が選択され、generation 1320 まで継続描画した。libEGL / ZINK の警告は発生したが、wgpu Adapter の生成、Dock 内表示、継続描画は成立したため検証失敗とは扱わない。\n\n### 評価対象外

- zero-copy / GPU 間 Texture 共有の最適化
- 正式 Surface API
- Runtime 統合
- IDE 統合

### 判定

Windows 実機について WV-12-02 は完了とする。

Linux VM については WV-12-03 以降と合わせて同一 wgpu Surface 経路を追加検証する。

## WV-12-03 Resize / Visibility / Lifecycle 検証

### 目的

GPU Surface が Dock の状態変化に追従できることを確認する。

### 合格条件

- Dock サイズ変更に追従できる。
- 表示 / 非表示に追従できる。
- Surface の生成 / 破棄を制御できる。
- 再生成時にリソース破棄漏れがない見込みを判断できる。

### Windows / Linux VM 検証結果\n\n| 動作対象 | Resize | Visibility | 生成 / 破棄 | 再生成時リソース管理 |\n| --- | --- | --- | --- | --- |\n| Windows 実機 | ○ | ○ | ○ | ○ |\n| Linux VM | ○ | ○ | ○ | ○ |\n| Linux 実機 | ？ | ？ | ？ | ？ |\n| macOS | ？ | ？ | ？ | ？ |\n\nLinux VM では Dock サイズ変更に応じて Texture の破棄 / 再生成が繰り返され、instance 1 から 19 まで生成・破棄後も描画継続を確認した。補足検証では `GPU Surface visibility: hidden` / `shown` が記録され、Show 後も同一 instance で generation が増加した。\n\n明示的な Destroy / Create と再生成時の旧 egui Texture 登録解除および wgpu Texture drop 経路が成立している。これはリソース管理経路の成立確認であり、VRAM leak が存在しないことを保証するものではない。\n\n## WV-12-04 入力イベント転送検証

### 目的

Dock 上の入力イベントを GPU Surface 側へ転送できることを確認する。

### 合格条件

- マウス移動を転送できる。
- マウスクリックを転送できる。
- ホイール入力を転送できる。
- キーボード入力を転送できる。
- Focus 状態を管理できる。

### Windows / Linux VM 検証結果\n\n| 動作対象 | PointerMove | PointerButton | Wheel | Keyboard / Text | Focus |\n| --- | --- | --- | --- | --- | --- |\n| Windows 実機 | ○ | ○ | ○ | ○ | ○ |\n| Linux VM | ○ | ○ | ○ | ○ | ○ |\n| Linux 実機 | ？ | ？ | ？ | ？ | ？ |\n| macOS | ？ | ？ | ？ | ？ | ？ |\n\nLinux VM の初回検証では PointerMove=1050、PointerButton=3、Keyboard=12、Focus=3 を確認した。Wheel は初回操作されなかったため補足検証を行い、上下方向の Wheel event を 16 件確認した。これにより Linux VM の基本入力5項目をすべて `○` とする。\n\n## WV-12-05 Browser Surface との比較整理

### 目的

Browser Surface と GPU Surface の共通化可能な要素、共通化すべきでない要素を整理する。

### 整理対象

- Texture / Frame 更新
- Resize
- Visibility
- Lifecycle
- 入力イベント
- エラー通知
- ブラウザ固有要素
- GPU 固有要素

### 完了条件

P0-2 完了後の仕様フェーズで Surface 共通 API を検討するための材料を整理できること。

## WV-12-06 GPU Surface 実験用インターフェース整理

### 目的

GPU Surface 技術検証で必要になった最小インターフェースを整理する。

本項目で整理するインターフェースは、技術検証用の実験的な内部インターフェースであり、Framework 利用アプリ向けの正式 API 仕様ではない。

### 位置付け

- GPU Surface の成立性を確認するための検証用インターフェースとする。
- Framework 内部での実装切り分けに使用する。
- 他 Dock Panel との整合は本項目では扱わない。
- Browser Surface との共通化は WV-12-05 の結果を踏まえて判断する。
- 正式 API 仕様は P0-2 完了後の仕様フェーズで定義する。

### 整理対象

- GPU Surface 作成
- 描画開始 / 終了
- Texture 更新
- リサイズ
- 入力イベント
- ライフサイクル
- エラー通知

### 完了条件

GPU Surface の技術検証に必要な最小インターフェースを整理し、正式仕様とは分離して記録できること。

### 判定\n\nWV-12-05 は [WV-12-05 Browser Surface と GPU Surface 比較整理](WV-12-05_Browser_SurfaceとGPU_Surface比較整理.md) に記録し、完了とする。\n\nWV-12-06 は [WV-12-06 GPU Surface 実験用インターフェース整理](WV-12-06_GPU_Surface実験用インターフェース整理.md) に記録し、完了とする。\n\n## WV-12 完了条件

WV-12 は以下を満たした時点で完了とする。

- Windows 上で GPU Surface が Dock 内に表示できる。
- GPU Surface がリサイズに追従できる。
- GPU Surface に基本入力を転送できる。
- Linux でも同方式を採用できる見込みを判断できる。
- Browser Surface と GPU Surface の共通化可能な要素、共通化すべきでない要素を整理できる。
- GPU Surface の技術検証用インターフェースを正式仕様と分離して整理できる。

## 次工程

WV-12-03 Resize / Visibility / Lifecycle 検証へ進む。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-12 GPU Surface成立性検証
