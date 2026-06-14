<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000016Z-W816
lang: ja-JP
canonical_title: WV-08-16 GTKイベントポンプ再導入検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-16 GTKイベントポンプ再導入検証

# WV-08-16 GTKイベントポンプ再導入検証

## 目的

GTKイベントポンプを再導入した場合に応答なしが発生するか確認する。

---

## 実施内容

* build_gtk()
* GtkFixed attach
* set_bounds継続実行
* set_visible(true)
* set_visible(false)
* flush_gtk_events_throttled() 再導入

---

## 判定条件

応答なし発生:

* GTKイベント処理が主因候補

応答なし未発生:

* GTKイベント処理を主因から除外

---

## 結果

* WV-08-16 throttled GTK flush request を確認
* WV-08-13 GTK event flush start label=WV-08-16 を確認
* WV-08-13 GTK event flush completed label=WV-08-16 を確認
* Dock移動可能: OK
* Dockサイズ変更可能: OK
* マウス操作可能: OK
* 応答なし発生せず

---

## 判定

成功

---

## 知見

* flush_gtk_events_throttled() を再導入しても応答なしは再現しなかった。
* GTKイベントポンプ単体は応答なしの主因ではない可能性が高い。
* WebView再配置、visibility制御、GTKイベントポンプの組み合わせでも応答なしは再現しなかった。

---

## 結論

GTKイベントポンプを再導入しても応答なしは再現しなかった。

これにより、以下はいずれも主因ではない可能性が高い。

* WebView::set_bounds() 継続実行
* WebView::set_visible(true)
* WebView::set_visible(false)
* flush_gtk_events_throttled()
* gtk::main_iteration_do(false)

次工程は Native Surface 表示切替経路の単独検証へ移行する。

---

次工程:

* WV-08-17 Native Surface表示切替単独検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-16 GTKイベントポンプ再導入検証
