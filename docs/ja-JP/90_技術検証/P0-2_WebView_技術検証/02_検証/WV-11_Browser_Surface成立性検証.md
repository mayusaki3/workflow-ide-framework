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

Windows / Linux / macOS を対象として、Dock 上で Web ブラウザ機能を Browser Surface として成立させられるか確認する。

手元で利用可能な環境から検証し、環境を用意できない対象は未検証として明示する。

本検証では、Framework 利用アプリ側に OS 依存コードを書かせず、Framework 側の Surface API が OS 差異を吸収できる構造を想定する。

## 背景

WV-10 までの検証により、Linux では GTK / WebKitGTK を生成・表示する構成で応答なしが発生することを確認した。

また、Wayland 環境では X11 reparent が成立せず、Windows 版の Child Window 相当の方式を Linux 主方式にできないことを確認した。

このため、WV-11 では Window 埋め込み方式ではなく、ブラウザ描画結果を Surface として取得し、Dock 内へ Texture として描画する方式を検証する。

## 前提

- 対象ブランチは `develop` とする。
- 対象 OS は Windows / Linux / macOS とする。
- Windows は現在利用可能な実機環境で検証する。
- Linux は現在利用可能な VM 環境で基本検証を行う。
- Linux 実機、X11 / Wayland、実 GPU、ドライバ等に依存する項目は、必要に応じて実環境で追加検証する。
- macOS を含め、検証環境を用意できない項目は未検証として扱い、非対応とはみなさない。
- 未検証環境については OSS の利点を生かし、検証手順と期待結果を公開して協力者を募る。
- P0-2 では Surface API の本仕様化、Runtime 統合、IDE 統合までは行わない。
- WV-11 内で整理するインターフェースは技術検証用であり、正式 API 仕様ではない。
- 正式 API 仕様は、WV-12 GPU Surface 技術検証および他 Dock Panel との整合を踏まえ、P0-2 完了後の仕様フェーズで定義する。

## 検証状態の表記

検証状態は、動作対象を縦軸、機能を横軸とした表で公開する。

- `○`: 検証済み・動作
- `×`: 検証済み・非動作
- `？`: 未検証

`×` は当該検証環境と検証時点で動作しなかったことを示し、Framework 全体として恒久的に非対応であることを意味しない。

VM と実機で結果に差が出る可能性がある場合は、同一 OS でも動作対象を別行として扱う。

## 検証方針

Browser Surface の主候補は CEF OSR とする。

CEF OSR により、ブラウザを独立 Window として表示せず、描画結果をバッファまたは Texture として取得し、egui / Dock 内へ表示する。

評価は以下の順で行う。

1. Browser Surface 方式選定
2. CEF OSR 最小構成検証
3. Browser Surface Texture 転送検証
4. 入力イベント転送検証
5. Windows 上の Dock 表示検証
6. Linux / macOS 上の成立性確認
7. Browser Surface 実験用インターフェース整理

## 検証項目

| 検証番号 | 項目 | 期待結果 |
| --- | --- | --- |
| WV-11-01 | Browser Surface 方式選定 | CEF OSR を採用候補として妥当か判断できる |
| WV-11-02 | CEF OSR 最小構成検証 | CEF OSR で描画バッファを取得できる |
| WV-11-03 | Browser Surface Texture 転送検証 | 取得した描画結果を egui Texture として Dock 内へ表示できる |
| WV-11-04 | 入力イベント転送検証 | egui 側の入力イベントを Browser Surface へ転送できる |
| WV-11-05 | Windows Dock 表示検証 | Windows 上で Browser Surface が Dock 内で動作する |
| WV-11-06 | Linux / macOS 成立性確認 | Linux / macOS 上で同方式を採用できる見込みを判断できる |
| WV-11-07 | Browser Surface 実験用インターフェース整理 | 技術検証に必要な最小インターフェースを正式仕様と分離して記録できる |

## WV-11-01 Browser Surface 方式選定

### 目的

Dock 上へ表示する Browser Surface の実装方式を選定する。

Framework 内部で Surface として扱えることを最優先とする。

### 評価条件

必須条件は以下とする。

- Windows 対応
- Linux 対応可能
- macOS 対応可能
- Off-Screen Rendering 対応
- 描画バッファ取得
- 入力イベント転送
- Texture 化可能
- Framework 側で Surface API へ統合可能

### 評価対象

| 方式 | Windows | Linux | macOS | OSR | 評価 |
| --- | --- | --- | --- | --- | --- |
| WebView2 | 可 | 不可 | 不可 | 不可 | Windows 専用のため共通方式から除外 |
| WebKitGTK | 不可 | 可 | 不可 | 不可 | WV-10 で Host Window 方式を終了 |
| CEF OSR | 可 | 可 | 可 | 可 | 主候補 |

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

### 検証基準 CEF

WV-11-02 の検証基準は stable 系の `151.3.24+g2384915+chromium-151.0.7922.174` とし、対応する Rust バインディングは `cef 151.8.1+151.3.24` を使用する。

検証結果には実際に使用した CEF バージョンを記録し、将来の Framework 実装で恒久固定するバージョンとは区別する。

### 合格条件

- CEF 初期化に成功する。
- Browser 作成に成功する。
- OSR モードで起動できる。
- Paint コールバックを受信できる。
- RGBA バッファを取得できる。
- 画面更新が継続する。
- Browser を終了し、CEF を正常に shutdown できる。

### 評価対象外

- Dock 表示
- egui Texture 転送
- GPU Surface
- Surface API
- 入力イベント

### CEF-IT-SPEC-001 CEF ランタイムロード
<!-- hldocs:sec_id=sec_10811wkg708i -->

#### 概要

CEF ランタイムを Framework の技術検証プロセスからロードできることを確認する。

#### 前提条件

- 対象 OS 用の CEF ランタイムが配置されている。
- 検証対象 CEF バージョンを識別できる。

#### 検証内容

- CEF ライブラリを指定配置または明示パスからロードする。

#### 期待結果

- CEF ライブラリのロードに成功する。
- 使用した CEF ライブラリのパスを識別できる。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_10811wkg708i

### CEF-IT-SPEC-002 必須シンボル解決
<!-- hldocs:sec_id=sec_pkl5oq8fkmc6 -->

#### 概要

WV-11-02 に必要な CEF C API シンボルを解決できることを確認する。

#### 前提条件

- CEF ランタイムのロードに成功している。

#### 検証内容

- 初期化、メッセージループ、Browser 作成、終了処理に必要な CEF シンボルを解決する。

#### 期待結果

- WV-11-02 で使用する必須シンボルをすべて解決できる。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_pkl5oq8fkmc6

### CEF-IT-SPEC-003 CEF 初期化
<!-- hldocs:sec_id=sec_aj1rfgg9rguj -->

#### 概要

CEF の初期化処理が成功することを確認する。

#### 前提条件

- CEF ランタイムのロードと必須シンボル解決に成功している。
- 使用する CEF バージョンと一致する ABI 定義を使用している。

#### 検証内容

- プラットフォームに必要な main args と最小 settings を構成し、CEF を初期化する。

#### 期待結果

- CEF 初期化が成功を返す。
- 初期化中に異常終了しない。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_aj1rfgg9rguj

### CEF-IT-SPEC-004 Windowless Browser 作成
<!-- hldocs:sec_id=sec_fjanz0cmlgpv -->

#### 概要

Native Window を表示せずに Browser を生成できることを確認する。

#### 前提条件

- CEF 初期化に成功している。

#### 検証内容

- Windowless Rendering を有効にした Browser を生成する。

#### 期待結果

- Browser 作成に成功する。
- Browser 用の独立 Native Window を表示しない。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_fjanz0cmlgpv

### CEF-IT-SPEC-005 Paint コールバック受信
<!-- hldocs:sec_id=sec_twsz1102y36h -->

#### 概要

OSR 描画更新を Paint コールバックとして受信できることを確認する。

#### 前提条件

- Windowless Browser の作成に成功している。
- 描画対象 URL が読み込まれている。

#### 検証内容

- CEF のメッセージ処理を継続し、Paint コールバック発生を観測する。

#### 期待結果

- Paint コールバックを1回以上受信できる。
- 描画領域の幅と高さを取得できる。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_twsz1102y36h

### CEF-IT-SPEC-006 描画バッファ取得
<!-- hldocs:sec_id=sec_brb8qsq5y25r -->

#### 概要

Paint コールバックから Browser Surface の描画元となる画素バッファを取得できることを確認する。

#### 前提条件

- Paint コールバックを受信できている。

#### 検証内容

- Paint コールバックで渡される画素バッファとサイズを取得する。

#### 期待結果

- 描画領域に対応する画素データを取得できる。
- 後続の Texture 転送検証で利用可能なバッファとして保持できる。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_brb8qsq5y25r

### CEF-IT-SPEC-007 継続描画更新
<!-- hldocs:sec_id=sec_sdnuof1lgo4n -->

#### 概要

Browser の画面更新に応じて OSR 描画更新が継続することを確認する。

#### 前提条件

- Paint コールバックから画素バッファを取得できている。

#### 検証内容

- 初回描画後も CEF のメッセージ処理を継続し、複数回の描画更新を観測する。

#### 期待結果

- Paint コールバックを複数回受信できる。
- 画面更新に応じて新しい画素バッファを取得できる。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_sdnuof1lgo4n

### CEF-IT-SPEC-008 Browser / CEF 終了
<!-- hldocs:sec_id=sec_y5zildrpcolm -->

#### 概要

Browser と CEF を正常な順序で終了できることを確認する。

#### 前提条件

- CEF 初期化後に Browser を生成している。

#### 検証内容

- Browser を終了し、必要なメッセージ処理完了後に CEF を shutdown する。

#### 期待結果

- Browser を正常に終了できる。
- CEF shutdown が完了する。
- 終了処理中に異常終了しない。

#### 参照仕様

- doc-20260524-009601Z-R8M7#sec_y5zildrpcolm

### WV-11-02 検証状態

| 動作対象 | Runtime | Symbols | Initialize | Browser | Paint | Buffer | Update | Shutdown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Windows 実機 | ○ | ○ | ○ | ○ | ○ | ○ | ？ | ？ |
| Linux VM | ？ | ？ | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux X11 実機 | ？ | ？ | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux Wayland 実機 | ？ | ？ | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Apple Silicon | ？ | ？ | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Intel | ？ | ？ | ？ | ？ | ？ | ？ | ？ | ？ |

## WV-11-03 Browser Surface Texture 転送検証

### 目的

CEF OSR の描画結果を egui の Texture として表示できることを確認する。

Browser Surface は Native Window を表示せず、Framework の Dock 上へ描画されることを目標とする。

### 合格条件

- OnPaint から取得した描画バッファを利用できる。
- egui Texture を生成できる。
- Texture を継続更新できる。
- Dock 内へ表示できる。
- Native Window を表示しない。

### 評価項目

- 描画バッファから Texture を作成できること。
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

## WV-11-06 Linux / macOS 成立性確認

### 目的

Linux / macOS 上で同方式を採用できる見込みを確認する。

検証環境を用意できない動作対象は未検証として残し、OSS 上で検証協力を募る。

### 合格条件

- Linux では GTK / WebKitGTK Host Window 方式に戻らず検証できる。
- Linux Wayland 上で Window 埋め込み方式を前提にしない。
- Linux で CEF OSR または同等方式により Surface 化できる見込みを判断できる。
- macOS で Window 埋め込み方式を前提とせず Surface 化できる構造であることを確認できる。
- 手元に検証環境がない項目は `？` として明示できる。

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

- 利用可能な検証環境で Browser Surface が Dock 内に表示できることを確認している。
- Browser Surface がリサイズに追従できる。
- Browser Surface に基本入力を転送できる。
- Windows / Linux / macOS の各動作対象について `○` / `×` / `？` の検証状態を公開できる。
- 検証環境がない動作対象を未検証として明示し、非対応と混同していない。
- Browser Surface の技術検証用インターフェースを正式仕様と分離して整理できる。

## 次工程

WV-11-02 CEF OSR 最小構成検証へ進む。

WV-11 完了後は、WV-12 GPU Surface 技術検証へ進む。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11 Browser Surface成立性検証