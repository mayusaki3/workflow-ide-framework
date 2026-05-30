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
| WV-00-04 | 成功 | Child Window配置に必要な座標情報取得成功 |

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

### 次アクション

PoC-1e:
Child Window生成

PoC-1f:
Dock追従確認

WV-01:
WebView表示

## 検証結果サマリー

| 検証番号 | 検証項目 | 結果 | 備考 |
|---|---|---|---|
| WV-01 | WebView 表示 | 未実施 |  |
| WV-02 | egui 共存 | 未実施 |  |
| WV-03 | Multi Panel | 未実施 |  |
| WV-04 | Focus 切替 | 未実施 |  |
| WV-05 | Rust から WebView への通知 | 未実施 |  |
| WV-06 | WebView から Rust への通知 | 未実施 |  |
| WV-07 | Command Bridge | 未実施 |  |

## WV-01 WebView 表示

### 実施結果

未実施。

### 確認内容

- WebView が表示されたか。
- 検証用 HTML が表示されたか。
- アプリケーションが異常終了しなかったか。

### 判定

未判定。

## WV-02 egui 共存

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

未記入。

## 判断

未判定。

### 次アクション

PoC-1e:
Child Window生成

PoC-1f:
Dock追従確認

WV-01:
WebView表示

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証結果
