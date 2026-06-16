<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260616-000096Z-W90605
lang: ja-JP
canonical_title: WV-09-06-05 Native Surface位置同期単独有効検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > [WV-09-06 実運用同期処理切り分け検証](WV-09-06_実運用同期処理切り分け検証.md) > WV-09-06-05 Native Surface位置同期単独有効検証

# WV-09-06-05 Native Surface位置同期単独有効検証

## 目的

Native Surface位置同期が Linux応答停止の発生条件に関係するか確認する。

## 検証方針

Native Surface の位置・サイズ同期は維持する。

継続的な GTK Window show / hide と WebView set_visible 呼び出しは実行せず、ログ出力のみ行う。

初期表示成立のため、初期化時の GTK Window表示処理は維持する。

## 検証構成

- GDK_BACKEND=x11
- eframe
- egui
- GTK Host Window
- Native Child Window
- WebKitGTK
- Native Surface位置同期有効
- WebView set_bounds有効
- GTKイベントポンプ有効
- GTK Window show / hide 継続実行停止
- WebView set_visible 呼び出し停止

## 検証項目

| 検証番号 | 項目 | 期待結果 |
|----------|------|----------|
| WV-09-06-05-01 | 起動確認 | 起動成功 |
| WV-09-06-05-02 | 初期表示確認 | Window表示確認 |
| WV-09-06-05-03 | Dockドラッグ | 表示維持確認 |
| WV-09-06-05-04 | Dock追従 | 位置・サイズ追従確認 |
| WV-09-06-05-05 | マウス移動 | 応答停止有無確認 |

## 判定基準

応答停止が再現しない場合、Native Surface位置同期単独を主因候補から除外する。

応答停止が再現する場合、Native Surface位置同期を主因候補とする。

Window表示またはDock追従が成立しない場合、当該構成は表示成立条件を満たさないため、不合格とする。

## 検証結果

実施完了。

観測結果は以下の通り。

- ビルド成功
- 起動成功
- gtk::init 成功
- WebView build_gtk 成功
- Native Child Window同期成功
- WebView set_bounds 成功
- Native Surface位置同期は継続実行される
- GTK Window show / hide は skip ログを出力し、実処理されない
- WebView set_visible は skip ログを出力し、実処理されない
- Window表示は成立する
- Dockドラッグ後も表示は維持される
- Window は Dock に合わせて移動する
- Window は Dock に合わせてサイズ変更される
- 応答なしは発生しない

## 判定

合格とする。

Native Surface位置同期を継続有効化し、GTK Window show / hide と WebView set_visible 呼び出しを停止しても、Window表示、Dock追従、サイズ変更は成立した。

また、応答なしは発生しなかった。

## 知見

Native Surface位置同期単独では Linux応答停止は再現しなかった。

このため、Native Surface位置同期は Linux応答停止の単独主因ではない。

## 結論

Native Surface位置同期単独を主因候補から除外する。

残存主因候補は以下とする。

- GTK Window show / hide と GTKイベントポンプの組み合わせ
- Visibility同期単独
- 複数同期処理の組み合わせ
- 現在の develop では再現条件を失っている可能性

## 次工程

WV-09-06 親文書へ結果を反映する。

その後、WV-09-06-06 Visibility同期単独有効検証の実施要否を判断する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > [WV-09-06 実運用同期処理切り分け検証](WV-09-06_実運用同期処理切り分け検証.md) > WV-09-06-05 Native Surface位置同期単独有効検証
