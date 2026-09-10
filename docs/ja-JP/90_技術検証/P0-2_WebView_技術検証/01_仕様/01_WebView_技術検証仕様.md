<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260524-009601Z-R8M7
lang: ja-JP
canonical_title: P0-2 WebView 技術検証仕様
document_type: spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 技術検証仕様

# WebView 技術検証仕様

## 1. 概要

P0-2 は、IDE の Dock 内でブラウザ描画を Browser Surface として扱える構造を対象とする。

Browser Surface は Native Window 埋め込みに依存せず、ブラウザ描画結果を Surface として取得し、Framework 側で扱える構造とする。

## 2. 対象

- 対象 OS は Windows / Linux / macOS とする。
- Framework 利用アプリ側へ OS 依存コードを要求しない構造とする。
- Browser Surface の主候補は CEF OSR とする。
- 検証環境を用意できない動作対象は未検証として扱い、非対応とはみなさない。
- P0-2 では正式 Surface API、Runtime 統合、IDE 統合の仕様確定は行わない。

## 3. Browser Surface 成立条件

### 3.1 CEF ランタイムロード
<!-- hldocs:sec_id=sec_10811wkg708i -->

対象 OS 用の CEF ランタイムを Framework 内部の技術検証境界からロード可能でなければならない。

### 3.2 CEF 必須シンボル
<!-- hldocs:sec_id=sec_pkl5oq8fkmc6 -->

Browser Surface の成立性確認に必要な CEF C API シンボルを Framework 内部から解決可能でなければならない。

### 3.3 CEF 初期化
<!-- hldocs:sec_id=sec_aj1rfgg9rguj -->

対象 CEF の ABI と一致する定義を使用し、CEF を正常に初期化可能でなければならない。

CEF のマルチプロセス動作に必要な subprocess 判定を `cef_execute_process` で初期化前に実行し、subprocess ではその戻り値に従ってアプリケーション本体の初期化処理へ進まず終了しなければならない。

メインの Browser Process のみが `cef_initialize` へ進み、初期化成功後に `cef_shutdown` を実行可能でなければならない。

### 3.4 Windowless Browser
<!-- hldocs:sec_id=sec_fjanz0cmlgpv -->

Browser Surface は、独立した Browser 用 Native Window を表示せずに Browser を生成可能でなければならない。

### 3.5 Paint 通知
<!-- hldocs:sec_id=sec_twsz1102y36h -->

Browser Surface は、OSR による描画更新を Paint 通知として取得可能でなければならない。

### 3.6 描画バッファ
<!-- hldocs:sec_id=sec_brb8qsq5y25r -->

Browser Surface は、Paint 通知から描画領域と画素バッファを取得可能でなければならない。

### 3.7 継続更新
<!-- hldocs:sec_id=sec_sdnuof1lgo4n -->

Browser Surface は、ブラウザ画面の変化に応じて描画バッファを継続更新可能でなければならない。

### 3.8 ライフサイクル
<!-- hldocs:sec_id=sec_y5zildrpcolm -->

Browser Surface の Browser と CEF は、必要な処理完了後に正常な順序で終了可能でなければならない。

## 4. OS 差異

OS 固有のランタイム配置、プロセス起動方式、描画バックエンドその他の差異は Framework 内部で吸収する。

利用可能な検証環境で成立性を確認し、環境がない動作対象は未検証状態として保持する。

## 5. 今後の拡張

- Browser Surface Texture 転送
- Browser Surface 入力転送
- GPU Surface との共通化
- Surface 共通 API
- Runtime 統合
- IDE 統合

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 技術検証仕様
