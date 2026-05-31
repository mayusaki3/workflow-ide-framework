<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000003Z-WV22
lang: ja-JP
canonical_title: WebView 検証結果
document_type: test_result
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証結果

# WebView 検証結果

## 目的

P0-2 WebView 技術検証の実施結果を記録する。

本結果は、`02_検証仕様/WebView_検証項目.md` に定義した WV-01 から WV-07 までの検証項目に対応する。

## 検証実施情報

| 項目 | 内容 |
|---|---|
| 検証日 | 未実施 |
| 検証者 | 未記入 |
| 対象ブランチ | develop |
| 対象コミット | 未記入 |
| 対象 OS | 未記入 |
| Rust バージョン | 未記入 |
| egui バージョン | 未記入 |
| eframe バージョン | 未記入 |
| wry バージョン | 未記入 |

## WV-00 Dock埋め込み成立性確認

### 実施結果

| 検証番号     | 結果 | 備考                         |
| -------- | -- | -------------------------- |
| WV-00-01 | 成功 | Dock Panel矩形取得成功           |
| WV-00-02 | 成功 | Dock移動検知成功                 |
| WV-00-03 | 成功 | Dockリサイズ検知成功               |
| WV-00-04 | 成功 | Child Window配置およびDock矩形追従成功 |
| WV-00-05 | 条件付き成功 | Child WindowはDock追従可能。ただしDock UI操作を阻害する |

### OS別確認結果

| OS             | 結果   | 備考                             |
| -------------- | ---- | ------------------------------ |
| Windows        | 成功   | ViewportInfo.outer_rect取得可能    |
| Ubuntu Desktop | 制約あり | ViewportInfo.outer_rect が None |

### 技術判断

ViewportInfo への依存は採用しない。

Child Window の配置計算は winit Window API を利用する。

### PoC-1a ViewportInfo調査

Windows:
outer_rect取得可能

Ubuntu Desktop:
outer_rect取得不可 (None)

結論:
ViewportInfo依存不可

### PoC-1b FrameInfo調査

取得内容:
cpu_usage のみ

Window座標取得:
不可

結論:
FrameInfo依存不可

### 採用方針

Window位置取得は
winit Window API を利用する。

### PoC-1c CreationContext調査

取得内容:

* egui_ctx
* storage
* integration_info

確認結果:

integration_info から取得できる情報は cpu_usage のみ。

Window座標取得:
不可

### 判断

CreationContext 経由で Window API へ到達する方式は採用しない。

Cargo 依存関係調査の結果、winit を直接利用可能であることを確認した。

今後は eframe から Window を取得するのではなく、winit を直接利用して Child Window を生成する方針とする。

### PoC-1d eframe Native Window取得調査

取得内容:

* ViewportId
* pixels_per_point

確認結果:

* `egui::ViewportId::ROOT` は取得可能
* ViewportId は `"FFFF"` として表示された
* `CreationContext` から Native Window Handle へ到達できない
* `eframe::Frame` から Native Window Handle へ到達できない
* Child Window生成に利用可能な親Window情報は取得できない

Window座標取得:
不可

Native Window Handle取得:
不可

### 判断

ViewportId は取得可能だが、Child Window生成の親Window指定には利用できない。

eframe公開APIから親Windowを取得する方式は採用しない。

今後は eframe から Window を取得するのではなく、winit / OS Native API を直接利用して Child Window 生成可否を確認する。

### PoC-1e Foreground Window取得調査

取得内容:

* GetForegroundWindow()

確認結果:

Windows:
* HWND取得成功

Ubuntu Desktop:
* Windows API利用不可

判断:

Windows環境では HWND を取得可能であることを確認した。

Child Window生成検証へ進む。

### PoC-1e-2 CreateWindowEx調査

取得内容:

* RegisterClassW
* CreateWindowExW
* ShowWindow

確認結果:

Windows:
* Window生成成功
* HWND取得成功
* Window表示成功

取得値例:

* RegisterClassW = 49875
* HWND(0x1241462)

判断:

Windows API による独立ネイティブWindow生成は可能。

次工程:
SetParent による親子Window化確認

### PoC-1e-3 SetParent調査

取得内容:

* SetParent
* IsWindow

確認結果:

Windows:

* Parent HWND取得成功
* Child HWND生成成功
* Parent != Child確認
* SetParent成功
* Child Window表示成功
* IsWindow=true

取得値例:

* Parent HWND = HWND(0x401104)
* Child HWND = HWND(0x2810122)

判断:

Windows APIによるChild Window化は可能。

WV-00の前提条件は成立した。

次工程:
PoC-1f Dock追従確認

### PoC-1f Dock追従確認

取得内容:

* MoveWindow
* Dock Panel Rect
* Child Window追従

確認結果:

Windows:

* Child WindowをDock Panel矩形へ追従可能
* Dock移動時に追従確認
* Dockリサイズ時に追従確認
* レイアウト変更後も再配置可能

確認結果:

* Overlay方式によりDock埋め込み相当の表示は可能
* Child WindowがDock UIのマウス操作を阻害する

追加検証:

Hide Child Window
↓
Dock操作
↓
Show Child Window

結果:

* Dock操作正常
* Child Window正常復帰

判断:

Child Window Overlay方式は成立する。

ただしDock UIとの共存に課題がある。

Dock操作中のみWebViewを非表示化する方式は成立可能性が高い。

### 次アクション

PoC-2e:
Dock Panel配置確認

WV-02:
egui共存確認

## 検証結果サマリー

| 検証番号 | 検証項目 | 結果 | 備考 |
|---|---|---|---|
| WV-01 | WebView 表示 | 成功 | build_as_child(frame) により表示成功 |
| WV-02 | egui 共存 | 未実施 |  |
| WV-03 | Multi Panel | 未実施 |  |
| WV-04 | Focus 切替 | 未実施 |  |
| WV-05 | Rust から WebView への通知 | 未実施 |  |
| WV-06 | WebView から Rust への通知 | 未実施 |  |
| WV-07 | Command Bridge | 未実施 |  |

## WV-01 WebView 表示

### 実施結果

### PoC-2a wry導入確認

取得内容:

* Cargo.toml
* wry 0.53.5
* webview2-com

確認結果:

Windows:

* cargo build 成功
* wry導入成功
* webview2-com導入成功
* eframe 0.33.0 と共存可能

判断:

WV-01を継続可能。

依存関係競合は確認されなかった。

### PoC-2b build_as_child調査

取得内容:

* WebViewBuilder
* build_as_child

確認結果:

Windows:

* build_as_child 存在確認
* HWND は HasWindowHandle を実装していない

確認結果:

* HWND → build_as_child は利用不可
* Child Window HWND を直接親として利用できない

判断:

HWND直結方式は採用しない。

wry が要求する Window Handle 抽象化を利用する方式へ移行する。

### PoC-2c WebViewBuilder調査

取得内容:

* WebViewBuilder::new()
* with_url()

確認結果:

Windows:

* WebViewBuilder::new() 成功
* with_url("https://example.com") 成功
* cargo run 成功

ログ:

WV-01 create start
WV-01 builder created
WV-01 PoC-2c ready

確認結果:

* Builder生成成功
* URL設定成功
* 実行時異常なし

判断:

wry API利用可能。

WV-01の前提条件は成立した。

次工程:

PoC-2d 実WebView生成確認

### PoC-2d HasWindowHandle調査

目的:

* eframe が提供する HasWindowHandle 実装を利用して WebView を生成可能か確認する

確認内容:

* build_as_child()
* build()
* eframe::Frame 利用可否
* WebView生成可否

成功条件:

* WebView生成成功
* example.com表示成功

失敗条件:

* HasWindowHandle を利用した WebView生成不可

確認結果:

Windows:

* build_as_child(frame) 成功
* WebView生成成功
* example.com表示成功
* アプリケーション異常終了なし

ログ:

WV-01 create start
WV-01 WebView create success

判断:

eframe::Frame を利用した WebView生成は可能。

HWND を利用しない方式で WebView表示が成立した。

次工程:

PoC-2e Dock Panel配置確認

### 検証ケース

| 検証番号 | 結果 | 備考 |
|---|---|---|
| WV-01-01 | 成功 | WebView生成成功 |
| WV-01-02 | 成功 | example.com表示成功 |
| WV-01-03 | 成功 | アプリケーション異常終了なし |
| WV-01-04 | 成功 | build_as_child(frame) 成功 |

### 確認内容

- WebView が表示されたか。
- 検証用 HTML が表示されたか。
- アプリケーションが異常終了しなかったか。

### 判定

成功。

以下を確認した。

* build_as_child(frame) 成功
* WebView生成成功
* example.com表示成功
* アプリケーション異常終了なし

WV-01は成功と判定する。

## WV-02 egui 共存

### PoC-2e Dock配置確認

目的:

* WebViewをDock Panelへ配置可能か確認する

確認内容:

* Dock移動
* Dockリサイズ
* Dockタブ切替
* WebView再配置

成功条件:

* WebViewがDock操作に追従する
* アプリケーションが異常終了しない

### 実施結果

未実施。

### 確認内容

- egui UI が操作可能だったか。
- WebView UI が操作可能だったか。
- 片方の操作がもう片方の操作を阻害しなかったか。

### 判定

未判定。

## WV-03 Multi Panel

### 実施結果

未実施。

### 確認内容

- 複数 WebView を同時表示できたか。
- WebView ごとに異なる内容を表示できたか。
- 一方の操作が他方の状態を破壊しなかったか。

### 判定

未判定。

## WV-04 Focus 切替

### 実施結果

未実施。

### 確認内容

- egui から WebView へ Focus を切り替えられたか。
- WebView から egui へ Focus を切り替えられたか。
- 複数 WebView 間で Focus を切り替えられたか。
- キーボード入力が意図した対象に反映されたか。

### 判定

未判定。

## WV-05 Rust から WebView への通知

### 実施結果

未実施。

### 確認内容

- Rust 側から WebView 側へ通知できたか。
- WebView 側の表示または状態が更新されたか。
- 通知時に異常終了しなかったか。

### 判定

未判定。

## WV-06 WebView から Rust への通知

### 実施結果

未実施。

### 確認内容

- WebView 側から Rust 側へ通知できたか。
- Rust 側でメッセージを受信できたか。
- 受信内容をログまたは状態として確認できたか。

### 判定

未判定。

## WV-07 Command Bridge

### 実施結果

未実施。

### 確認内容

- WebView 側から Command 名と引数を送信できたか。
- Rust 側で Command として解釈できたか。
- Command 実行結果を確認できたか。

### 判定

未判定。

## OS 別差異

| OS | 結果 | 差異・制約 |
|---|---|---|
| Windows | 未実施 |  |
| Linux | 未実施 |  |
| macOS | 未実施 |  |

## 課題

### 課題-01

wry 0.53.5 の build_as_child は HasWindowHandle を要求する。

HWND は HasWindowHandle を実装していないため、
PoC-1e で生成した Child Window HWND を直接利用できない。

### 対応方針

* HWND直結方式は採用しない
* WRY Overlay方式を継続調査
* Windows / Linux / macOS 共通方式を優先する

## 判断

以下を確認した。

* Dock Panel矩形取得
* Dock移動検知
* Dockリサイズ検知
* ViewportInfo調査
* FrameInfo調査
* CreationContext調査
* eframe Native Window取得調査

結果として、eframe公開APIから親Window取得は確認できなかった。

WV-00について以下を確認した。

* Dock Panel矩形取得
* Dock移動検知
* Dockリサイズ検知
* Child Window生成
* Child Window親子化
* Child Window Dock追従

結果:

Child Window Overlay方式は成立した。

ただしDock UIとの同時操作ではマウス入力競合が発生する。

対策案:

* Dock操作中のみWebView非表示
* Dock操作終了後再表示

Hide/Show方式により正常復帰することを確認した。

WV-00は条件付き成功と判定する。

次工程:

WV-01 WebView表示

### 次アクション

PoC-2e:
Dock Panel配置確認

WV-02:
egui共存確認

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証結果
