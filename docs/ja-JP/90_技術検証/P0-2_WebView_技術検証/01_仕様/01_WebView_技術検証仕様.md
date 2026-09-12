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

### 3.9 Texture 画素形式
<!-- hldocs:sec_id=sec_3n8qk2v5u7am -->

Browser Surface は、CEF OSR が提供する BGRA 画素バッファを egui Texture が解釈可能な RGBA8 画素列へ正規化可能でなければならない。

### 3.10 egui Texture 生成
<!-- hldocs:sec_id=sec_r8m4t2x9c6kp -->

Browser Surface は、正規化した描画バッファと描画領域サイズから egui の Texture を生成可能でなければならない。

### 3.11 Texture 更新
<!-- hldocs:sec_id=sec_q6f1w9n3z8bd -->

Browser Surface は、CEF OSR の新しい Paint 通知に応じて既存の egui Texture を更新可能でなければならない。

### 3.12 Dock Panel 表示
<!-- hldocs:sec_id=sec_h5v2c8m7p1rs -->

Browser Surface は、Browser 用の独立 Native Window を表示せず、egui Texture として Dock Panel 内へ表示可能でなければならない。

### 3.13 描画サイズ同期
<!-- hldocs:sec_id=sec_b4j7k1s9d3wx -->

Browser Surface は、Dock Panel の表示領域変更に応じて Browser の OSR 描画領域と Texture のサイズを同期可能でなければならない。

### 3.14 表示継続更新
<!-- hldocs:sec_id=sec_m9a2e6r4t7yc -->

Browser Surface は、ブラウザ画面の変化に応じた継続的な Paint 更新を Dock Panel 内の表示へ反映可能でなければならない。

### 3.15 Pointer 座標変換
<!-- hldocs:sec_id=sec_v4c8n2q7m1px -->

Browser Surface は、Dock Panel 内で取得した Pointer 座標を Browser OSR の論理表示領域に対応する座標へ変換可能でなければならない。

### 3.16 Pointer 移動入力
<!-- hldocs:sec_id=sec_k7w3f9r2d6ta -->

Browser Surface は、Dock Panel 内の Pointer 移動を Browser へ転送可能でなければならない。

### 3.17 Pointer Button 入力
<!-- hldocs:sec_id=sec_p2m8x5c1q7vz -->

Browser Surface は、Dock Panel 内の Pointer Button の押下および解放を Browser へ転送可能でなければならない。

### 3.18 Wheel 入力
<!-- hldocs:sec_id=sec_a6t1n9w4k3rb -->

Browser Surface は、Dock Panel 内で発生した Wheel 入力を Browser へ転送可能でなければならない。

### 3.19 Keyboard 入力
<!-- hldocs:sec_id=sec_r5q9d2m8v1kc -->

Browser Surface は、Browser Surface が入力対象である間、Keyboard の押下および解放を Browser へ転送可能でなければならない。

### 3.20 Text 入力
<!-- hldocs:sec_id=sec_c3x7p1t9m5wf -->

Browser Surface は、Browser Surface が入力対象である間、文字入力を Browser の編集可能要素へ反映可能でなければならない。

## 4. OS 差異

OS 固有のランタイム配置、プロセス起動方式、描画バックエンドその他の差異は Framework 内部で吸収する。

利用可能な検証環境で成立性を確認し、環境がない動作対象は未検証状態として保持する。

## 5. 今後の拡張

- GPU Surface との共通化
- Surface 共通 API
- Runtime 統合
- IDE 統合

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 技術検証仕様
