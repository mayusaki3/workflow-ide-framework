<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260628-000011Z-WV11
lang: ja-JP
canonical_title: WV-11 Browser Surface成立性検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11 Browser Surface成立性検証

# WV-11 Browser Surface成立性検証

## 目的

Windows を主対象として、Dock 上で Web ブラウザ機能を Browser Surface として成立させられるか確認する。

Linux は次優先として、同方式を採用できる見込みを確認する。

本検証では、Framework 利用アプリ側に OS 依存コードを書かせず、Framework 側の Surface API が OS 差異を吸収できる構造を想定する。

## 背景

WV-10 までの検証により、Linux では GTK / WebKitGTK を生成・表示する構成で応答なしが発生することを確認した。

また、Wayland 環境では X11 reparent が成立せず、Windows 版の Child Window 相当の方式を Linux 主方式にできないことを確認した。

このため、WV-11 では Window 埋め込み方式ではなく、ブラウザ描画結果を Surface として取得し、Dock 内へ Texture として描画する方式を検証する。

## 前提

- 対象ブランチは `develop` とする。
- Windows を最優先対象とする。
- Linux は次優先対象とし、同方式の成立性を確認する。
- macOS は現時点では検証環境がないため、P0-2 では対象外とする。
- ただし、将来 macOS 実装を追加できる Surface 共通アーキテクチャを維持する。
- P0-2 では Surface API の本仕様化、Runtime 統合、IDE 統合までは行わない。
- WV-11 内で整理するインターフェースは技術検証用であり、正式 API 仕様ではない。
- 正式 API 仕様は、WV-12 GPU Surface 技術検証および他 Dock Panel との整合を踏まえ、P0-2 完了後の仕様フェーズで定義する。

## 検証方針

Browser Surface の主候補は CEF OSR とする。

CEF OSR により、ブラウザを独立 Window として表示せず、描画結果をバッファまたは Texture として取得し、egui / Dock 内へ表示する。

評価は以下の順で行う。

1. Browser Surface 方式選定
2. CEF OSR 最小構成検証
3. Browser Surface Texture 転送検証
4. 入力イベント転送検証
5. Windows 上の Dock 表示検証
6. Linux 上の成立性確認
7. Browser Surface 実験用インターフェース整理

## 検証項目

| 検証番号 | 項目 | 期待結果 |
| --- | --- | --- |
| WV-11-01 | Browser Surface 方式選定 | CEF OSR を採用候補として妥当か判断できる |
| WV-11-02 | CEF OSR 最小構成検証 | CEF OSR で描画バッファを取得できる |
| WV-11-03 | Browser Surface Texture 転送検証 | 取得した描画結果を egui Texture として Dock 内へ表示できる |
| WV-11-04 | 入力イベント転送検証 | egui 側の入力イベントを Browser Surface へ転送できる |
| WV-11-05 | Windows Dock 表示検証 | Windows 上で Browser Surface が Dock 内で動作する |
| WV-11-06 | Linux 成立性確認 | Linux 上でも同方式を採用できる見込みを判断できる |
| WV-11-07 | Browser Surface 実験用インターフェース整理 | 技術検証に必要な最小インターフェースを正式仕様と分離して記録できる |

## WV-11-01 Browser Surface 方式選定

### 目的

Dock 上へ表示する Browser Surface の実装方式を選定する。

Framework 内部で Surface として扱えることを最優先とする。

### 評価条件

必須条件は以下とする。

- Windows 対応
- 将来 Linux 対応可能
- Off-Screen Rendering 対応
- 描画バッファ取得
- 入力イベント転送
- Texture 化可能
- Framework 側で Surface API へ統合可能

### 評価対象

| 方式 | Windows | Linux | OSR | 評価 |
| --- | --- | --- | --- | --- |
| WebView2 | 可 | 不可 | 不可 | Windows 専用のため共通方式から除外 |
| WebKitGTK | 不可 | 可 | 不可 | WV-10 で Host Window 方式を終了 |
| CEF OSR | 可 | 可 | 可 | 主候補 |

### 調査結果

CEF は Windows / Linux / macOS で利用可能な Chromium ベースの埋め込みフレームワークである。

CEF 本体は C / C++ を主対象とするため、Rust から利用する場合は以下のいずれかを検討する。

- 既存 Rust バインディングまたはラッパーを評価する。
- 既存ラッパーで OSR / OnPaint / 入力転送を扱えない場合、CEF C API への FFI 境界を Framework 内部に閉じ込める。
- Framework 利用アプリには CEF 依存を公開せず、Browser Surface API のみを公開する。

現時点の判断では、WebView2 は Windows 専用、WebKitGTK は WV-10 の結果により Linux 主方式から外すため、Browser Surface の主候補は CEF OSR とする。

### 判定

WV-11-01 は完了とする。

Browser Surface 方式は CEF OSR を主候補として次工程へ進める。

### 完了条件

Browser Surface の主候補を CEF OSR として扱ってよいか判断できること。

## WV-11-02 CEF OSR 最小構成検証

### 目的

CEF OSR が Browser Surface の描画元として利用可能か確認する。

本検証では Dock、egui、GPU Surface、Surface API は対象外とし、CEF 単体で描画バッファを取得できることを確認する。

### 合格条件

- CEF 初期化に成功する。
- Browser 作成に成功する。
- OSR モードで起動できる。
- Paint コールバックを受信できる。
- RGBA バッファを取得できる。
- 画面更新が継続する。

### 評価対象外

- Dock 表示
- egui Texture 転送
- GPU Surface
- Surface API
- 入力イベント

## WV-11-03 Browser Surface Texture 転送検証

### 目的

CEF OSR の描画結果を egui の Texture として表示できることを確認する。

Browser Surface は Native Window を表示せず、Framework の Dock 上へ描画されることを目標とする。

### 合格条件

- OnPaint から取得した RGBA バッファを利用できる。
- egui Texture を生成できる。
- Texture を継続更新できる。
- Dock 内へ表示できる。
- Native Window を表示しない。

### 評価項目

- RGBA から Texture を作成できること。
- Texture を更新できること。
- Dock Panel 内へ描画できること。
- Dock サイズ変更へ追従できること。
- スクロール等で描画更新が継続すること。

### 評価対象外

- マウス入力
- キーボード入力
- IME
- Drag and Drop

## WV-11-04 入力イベント転送検証

### 目的

egui 側で受け取った入力イベントを Browser Surface へ転送できることを確認する。

### 合格条件

- マウス移動を転送できる。
- マウスクリックを転送できる。
- ホイール入力を転送できる。
- キーボード入力を転送できる。
- Focus 状態を管理できる。

## WV-11-05 Windows Dock 表示検証

### 目的

Windows 上で Browser Surface が Dock 内で動作することを確認する。

### 合格条件

- Browser Surface が Dock 内に表示される。
- Dock 移動、リサイズ、タブ切替に追従する。
- 基本入力が利用できる。
- 利用アプリから OS 非依存 API として扱える見込みを確認できる。

## WV-11-06 Linux 成立性確認

### 目的

Linux 上で同方式を採用できる見込みを確認する。

### 合格条件

- GTK / WebKitGTK Host Window 方式に戻らず検証できる。
- Wayland 上で Window 埋め込み方式を前提にしない。
- CEF OSR または同等方式で Surface 化できる見込みを判断できる。

## WV-11-07 Browser Surface 実験用インターフェース整理

### 目的

Browser Surface 技術検証で必要になった最小インターフェースを整理する。

本項目で整理するインターフェースは、技術検証用の実験的な内部インターフェースであり、Framework 利用アプリ向けの正式 API 仕様ではない。

### 位置付け

- Browser Surface の成立性を確認するための検証用インターフェースとする。
- Framework 内部での実装切り分けに使用する。
- 他 Dock Panel との整合は本項目では扱わない。
- GPU Surface との共通化は WV-12 の結果を踏まえて判断する。
- 正式 API 仕様は P0-2 完了後の仕様フェーズで定義する。

### 整理対象

- Browser Surface 作成
- URL 読み込み
- リサイズ
- 入力イベント
- Texture 更新
- ライフサイクル
- エラー通知

### 完了条件

Browser Surface の技術検証に必要な最小インターフェースを整理し、正式仕様とは分離して記録できること。

## WV-11 完了条件

WV-11 は以下を満たした時点で完了とする。

- Windows 上で Browser Surface が Dock 内に表示できる。
- Browser Surface がリサイズに追従できる。
- Browser Surface に基本入力を転送できる。
- Linux でも同方式を採用できる見込みを判断できる。
- Browser Surface の技術検証用インターフェースを正式仕様と分離して整理できる。

## 次工程

WV-11-02 CEF OSR 最小構成検証へ進む。

WV-11 完了後は、WV-12 GPU Surface 技術検証へ進む。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11 Browser Surface成立性検証
