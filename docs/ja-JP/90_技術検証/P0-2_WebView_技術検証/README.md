<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009501Z-Q4K1
lang: ja-JP
canonical_title: P0-2 WebView 技術検証
document_type: note
canonical_document: true
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-2 WebView 技術検証

# P0-2 WebView 技術検証

## 1. 目的

IDE の Dock 上で Browser Surface / GPU Surface を Native Child Window 埋め込みに依存せず扱える、Surface 共通アーキテクチャの技術成立性を確認する。

Browser Surface は CEF OSR、GPU Surface は wgpu を主検証方式とし、Framework 利用アプリ側へ OS / Runtime / GPU Backend 固有処理を直接要求しない構造の見込みを確認する。

## 2. 検証結果

P0-2 の技術成立性確認は完了した。

- Browser Surface: CEF OSR -> egui Texture -> Dock の経路が Windows 実機および Linux VM / Wayland で成立した。
- GPU Surface: wgpu -> GPU Texture -> egui native Texture -> Dock の経路が Windows 実機および Linux VM / Wayland で成立した。
- Windows 実機では Browser Surface の基本入力と追加 IME 検証、GPU Surface の Resize / Visibility / Lifecycle / 基本入力が成立した。
- Linux VM では Browser Surface の CEF OSR / Dock 表示 / Resize、GPU Surface の Dock 表示 / Resize / Visibility / Lifecycle / 基本入力が成立した。
- Linux VM の GPU Surface は llvmpipe / Vulkan による software renderer での成立確認であり、Linux 物理 GPU / Driver 差異は未検証である。
- Linux 実機および macOS は検証環境がない項目を `？` とし、非対応とは判定しない。
- Browser Surface / GPU Surface の共通化候補と固有責務を整理し、実験用内部インターフェースを正式 API と分離して記録した。

過去の WV-00～10 で検証した Native Window / GTK / WebKitGTK Host Window 系の方式は経緯として保持する。最終的な Surface 方式は WV-11 / WV-12 の結果を基準とする。

## 3. 残存事項

技術検証完了後も、以下は正式仕様または後続実装で扱う。

- Surface 共通 trait / struct / enum の正式定義
- async / sync と Runtime 所有モデル
- Browser / GPU 描画成果物を統一する adapter / output model
- GPU Device / Queue の共有単位
- zero-copy / GPU 間 Texture 共有などの性能最適化
- Linux 物理 GPU / Driver 差異の検証
- Linux 実機 / macOS の未検証項目
- Windows Browser Surface IME の最初の Space で Candidate Window が一度閉じて再表示される補足残課題
- Drag and Drop / Clipboard / Accessibility / security / sandbox 等の正式な責務境界

これらは P0-2 の技術成立条件を阻害するものとしては扱わない。

## 4. ドキュメント構成

### 仕様

- [WebView 技術検証仕様](01_仕様/01_WebView_技術検証仕様.md)

### 検証内容 / 検証結果

- [検証目次](./02_検証/検証目次.md)
- [WV-11 Browser Surface成立性検証](./02_検証/WV-11_Browser_Surface成立性検証.md)
- [WV-12 GPU Surface成立性検証](./02_検証/WV-12_GPU_Surface成立性検証.md)
- [WV-12-05 Browser SurfaceとGPU Surface比較整理](./02_検証/WV-12-05_Browser_SurfaceとGPU_Surface比較整理.md)
- [WV-12-06 GPU Surface実験用インターフェース整理](./02_検証/WV-12-06_GPU_Surface実験用インターフェース整理.md)

## 5. P0-2 判定

Windows を主対象とした Browser Surface / GPU Surface の成立、および Linux VM での同方式の基本成立を確認した。

Native Child Window / GTK Host Window を Surface 共通アーキテクチャの前提とせず、描画成果物を Dock 側へ接続し、Resize / Input / Focus / Lifecycle 等を Surface 境界として整理できる技術的見込みが得られた。

以上により P0-2 WebView 技術検証は完了とする。

正式 Surface API、Runtime 統合、IDE 統合は後続の仕様・実装工程で扱う。

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > P0-2 WebView 技術検証
