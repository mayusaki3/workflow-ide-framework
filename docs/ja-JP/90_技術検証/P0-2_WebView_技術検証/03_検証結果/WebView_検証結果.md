<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260531-000001Z-WVR0
lang: ja-JP
canonical_title: WebView 検証結果
document_type: test_result
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証結果

# WebView 検証結果

## 目的

P0-2 WebView 技術検証の実施結果を集約する。

詳細な検証過程は、検証番号ごとの結果ファイルへ分割して管理する。

## 検証実施情報

| 項目 | 内容 |
|---|---|
| 対象ブランチ | develop |
| 対象 OS | Windows / Linux / macOS |
| Rust バージョン | 未記入 |
| egui バージョン | 0.33.0 |
| eframe バージョン | 0.33.0 |
| wry バージョン | 0.53.5 |

## 検証結果サマリー

| 検証番号 | 検証項目 | 結果 | 備考 |
|---|---|---|---|
| WV-00 | Dock埋め込み成立性確認 | 条件付き成功 | Child Window Overlay方式は成立。ただしDock UI操作と入力競合あり |
| WV-01 | WebView表示 | 成功 | build_as_child(frame) により表示成功 |
| WV-02 | egui共存 | 実施中 | PoC-2e Dock配置確認へ移行 |
| WV-03 | Multi Panel | 未実施 |  |
| WV-04 | Focus切替 | 未実施 |  |
| WV-05 | Rust→WebView通知 | 未実施 |  |
| WV-06 | WebView→Rust通知 | 未実施 |  |
| WV-07 | Command Bridge | 未実施 |  |

## 詳細結果

- [WV-00_Dock埋め込み成立性確認.md](./WV-00_Dock埋め込み成立性確認.md)
- [WV-01_WebView表示.md](./WV-01_WebView表示.md)
- [WV-02_egui共存.md](./WV-02_egui共存.md)

## WV-03以降

WV-03 から WV-07 は未実施のため、検証結果ファイルは未作成。

## OS別差異

| OS | 結果 | 差異・制約 |
|---|---|---|
| Windows | 一部成功 | WV-00, WV-01確認済 |
| Linux | 未確認 |  |
| macOS | 未確認 |  |

## 現在の課題

### 課題-01

wry 0.53.5 の `build_as_child()` は `HasWindowHandle` を要求する。

HWND は `HasWindowHandle` を実装していないため、PoC-1e で生成した Child Window HWND は直接利用できない。

### 課題-02

`build_as_child(frame)` は Root Window 上に WebView を生成する。

Dock Panel には配置されないため、Dock移動・Dockレイアウト変更へ追従しない。

## 判断

WV-00では、Child Window Overlay方式によりDock矩形への追従は成立した。

WV-01では、`build_as_child(frame)` により WebView 表示が成立した。

次工程では、WV-02として egui Dock と WebView の共存性を確認する。

## 次アクション

PoC-2e:
Dock Panel配置確認

WV-02:
egui共存確認

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証結果
