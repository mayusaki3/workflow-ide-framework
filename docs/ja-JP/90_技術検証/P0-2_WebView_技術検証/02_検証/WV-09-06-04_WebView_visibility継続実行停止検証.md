<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260616-000096Z-W90604
lang: ja-JP
canonical_title: WV-09-06-04 WebView visibility継続実行停止検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > [WV-09-06 実運用同期処理切り分け検証](WV-09-06_実運用同期処理切り分け検証.md) > WV-09-06-04 WebView visibility継続実行停止検証

# WV-09-06-04 WebView visibility継続実行停止検証

## 目的

WV-09-06-03 の結果を受け、Visibility同期フェーズにおける WebView `set_visible` 継続実行が Linux応答停止の発生条件に関係するか確認する。

## 背景

WV-09-06-03 では、Visibility同期フェーズの GTKイベントポンプを停止しても、Window表示、Dock追従、サイズ変更が成立し、応答なしは発生しなかった。

この結果から、Visibility同期フェーズで継続実行される以下の組み合わせが主因候補として残った。

- WebView `set_visible(true)` / `set_visible(false)`
- GTKイベントポンプ
- GTK Window show / hide
- Native Surface位置同期

本検証では、GTKイベントポンプは有効に戻し、Visibility同期フェーズの WebView `set_visible` 呼び出しのみを停止する。

## 検証方針

Visibility同期処理自体は維持する。

ただし、Visibility同期フェーズで継続実行される WebView `set_visible` 呼び出しは実行せず、ログ出力のみ行う。

初回生成時の WebView `set_visible(true)` は維持する。

## 検証構成

- GDK_BACKEND=x11
- eframe
- egui
- GTK Host Window
- Native Child Window
- WebKitGTK
- Native Surface位置同期
- GTKイベントポンプ有効
- Visibility同期フェーズの WebView `set_visible` 継続実行のみ停止

## 検証項目

| 検証番号 | 項目 | 期待結果 |
|----------|------|----------|
| WV-09-06-04-01 | 起動確認 | 起動成功 |
| WV-09-06-04-02 | 初期表示確認 | Window表示確認 |
| WV-09-06-04-03 | Dockドラッグ | 表示維持確認 |
| WV-09-06-04-04 | Dock追従 | 位置・サイズ追従確認 |
| WV-09-06-04-05 | マウス移動 | 応答停止有無確認 |
| WV-09-06-04-06 | 長時間動作 | 応答停止有無確認 |

## 判定基準

応答停止が再現しない場合

- Visibility同期フェーズの WebView `set_visible` 継続実行を主因候補とする

応答停止が再現する場合

- GTKイベントポンプ単体、または GTK Window show / hide と GTKイベントポンプの組み合わせを主因候補とする

Window表示またはDock追従が成立しない場合

- 当該構成は表示成立条件を満たさないため、不合格とする

## 検証結果

未実施

## 判定

未判定

## 知見

なし

## 結論

未実施

## 次工程

検証結果を記録し、応答停止再現有無に応じて次の切り分けを決定する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > [WV-09-06 実運用同期処理切り分け検証](WV-09-06_実運用同期処理切り分け検証.md) > WV-09-06-04 WebView visibility継続実行停止検証
