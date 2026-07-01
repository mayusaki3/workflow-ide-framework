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

本検証では、GTKイベントポンプは有効に戻し、WebView `set_visible` 呼び出しを停止する。

## 検証方針

Visibility同期処理自体は維持する。

ただし、WebView `set_visible` 呼び出しは実行せず、ログ出力のみ行う。

GTK Window の `show_all()` / `hide()`、Native Surface位置同期、WebView `set_bounds` は維持する。

## 検証構成

- GDK_BACKEND=x11
- eframe
- egui
- GTK Host Window
- Native Child Window
- WebKitGTK
- Native Surface位置同期
- GTKイベントポンプ有効
- WebView `set_visible` 呼び出し停止

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

- WebView `set_visible` 継続実行を主因候補から除外する

応答停止が再現する場合

- GTKイベントポンプ単体、または GTK Window show / hide と GTKイベントポンプの組み合わせを主因候補とする

Window表示またはDock追従が成立しない場合

- 当該構成は表示成立条件を満たさないため、不合格とする

## 検証結果

実施完了。

観測結果は以下の通り。

- ビルド成功
- 起動成功
- gtk::init 成功
- WebView build_gtk 成功
- Native Child Window同期成功
- WebView set_bounds 成功
- WebView `set_visible(true)` は `skipped reason=visibility_call_disabled` を出力し、実行されない
- Dockドラッグ中も WebView `set_visible(true)` は `skipped reason=visibility_call_disabled` を出力し、実行されない
- Window表示は成立する
- Dockドラッグ後も表示は維持される
- Window は Dock に合わせて移動する
- Window は Dock に合わせてサイズ変更される
- 応答なしは発生しない

## 判定

合格とする。

WebView `set_visible` 呼び出しを停止しても、Window表示、Dock追従、サイズ変更は成立した。

また、応答なしは発生しなかった。

## 知見

WebView `set_visible` 継続実行を停止しても応答なしは再現しなかった。

このため、WebView `set_visible` 継続実行は Linux応答停止の単独主因ではない。

一方で、GTK Window `show_all()`、Native Surface位置同期、WebView `set_bounds`、GTKイベントポンプは維持された状態で応答なしが発生していないため、これら単独要因でも応答なしは再現していない。

## 結論

WebView `set_visible` 継続実行は主因候補から除外する。

残存主因候補は以下とする。

- Native Surface位置同期
- GTK Window show / hide
- GTKイベントポンプとの組み合わせ
- 現在の develop では再現条件を失っている可能性

## 次工程

WV-09-06 親文書へ結果を反映する。

その後、WV-09-06-05 Native Surface位置同期単独有効検証へ進む。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > [WV-09-06 実運用同期処理切り分け検証](WV-09-06_実運用同期処理切り分け検証.md) > WV-09-06-04 WebView visibility継続実行停止検証
