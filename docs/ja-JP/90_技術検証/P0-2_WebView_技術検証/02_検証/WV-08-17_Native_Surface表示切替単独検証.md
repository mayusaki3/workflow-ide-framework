<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000017Z-W817
lang: ja-JP
canonical_title: WV-08-17 Native Surface表示切替単独検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-17 Native Surface表示切替単独検証

# WV-08-17 Native Surface表示切替単独検証

## 目的

should_show_native_surface に連動した Native Surface 表示切替経路で、応答なしが発生するか確認する。

---

## 実施内容

* build_gtk()
* GtkFixed attach
* set_bounds継続実行
* set_visible(true)
* set_visible(false)
* flush_gtk_events_throttled() 継続
* should_show_native_surface に連動した Native Surface 表示切替

---

## 判定条件

応答なし発生:

* Native Surface 表示切替が主因候補

応答なし未発生:

* Native Surface 表示切替を主因から除外

---

## 結果

* WV-08-17 native surface show success を確認
* WV-08-17 native surface hide success を確認
* WV-02 Hide Native Surface 発生時に native surface hide success を確認
* Dock移動可能: OK
* Dockサイズ変更可能: OK
* マウス操作可能: OK
* 応答なし発生せず

補足:

* ウィンドウ上に表示されていた WV-08-13 ラベルは、Native Surface 非表示により表示されなくなった。
* Hide Native Surface 経路で WebView::set_visible(false) が実行され、その後 WebView::set_visible(true) へ復帰した。

---

## 判定

成功

---

## 知見

* should_show_native_surface=false の実運用経路で WebView::set_visible(false) が実行された。
* Native Surface 表示切替を継続しても応答なしは再現しなかった。
* WebView再配置、visibility制御、GTKイベントポンプ、Native Surface表示切替の組み合わせでも応答なしは再現しなかった。

---

## 結論

Native Surface 表示切替は応答なしの主因ではない可能性が高い。

WV-08 系で検証した GTK / WebKitGTK / WebView 操作系の要素では、応答なしを再現できなかった。

次工程は WV-09 系として、WV-07で応答なしが発生した構成との差分を再確認し、X11親子ウィンドウ制御、eframe / winit 共存、または実運用側の同期処理へ調査を移行する。

---

次工程:

* WV-09 系検証へ移行

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-17 Native Surface表示切替単独検証
