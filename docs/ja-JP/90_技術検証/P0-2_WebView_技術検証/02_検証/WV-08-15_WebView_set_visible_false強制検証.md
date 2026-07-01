<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000015Z-W815
lang: ja-JP
canonical_title: WV-08-15 WebView set_visible(false) 強制検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-15 WebView set_visible(false) 強制検証

# WV-08-15 WebView set_visible(false) 強制検証

## 目的

WebView::set_visible(false) を強制実行した場合に、応答なしが発生するか確認する。

---

## 実施内容

* WebKitGTK WebView を build_gtk() で生成する。
* WebView を static に保持する。
* Dock追従処理は維持する。
* WebView::set_bounds() 継続実行は維持する。
* sync_child_window() の先頭で WebView::set_visible(false) を強制実行する。
* set_visible(false) 実行後は return する。

---

## 判定条件

応答なし発生:

* WebView::set_visible(false) が主因候補

応答なし未発生:

* visibility制御は主因から除外

---

## 結果

* WV-08-15 force hide success を確認
* WebView::set_visible(false) が継続実行された
* Dock移動可能: OK
* Dockサイズ変更可能: OK
* マウス操作可能: OK
* 応答なし発生せず

---

## 判定

成功

---

## 知見

* WebView::set_visible(false) を継続実行しても応答なしは再現しなかった。
* visibility制御単体は応答なしの主因ではない可能性が高い。
* WebView::set_visible(true) に加え、WebView::set_visible(false) も除外候補となった。

---

## 結論

WebView::set_visible(false) を強制実行しても応答なしは再現しなかった。

これにより、

* WebView::set_visible(true)
* WebView::set_visible(false)

はいずれも主因ではない可能性が高い。

調査対象は GTKイベントポンプ、eframe / winit 共存、Native Surface表示切替へ移行する。

---

次工程:

* WV-08-16 GTKイベントポンプ再導入検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-15 WebView set_visible(false) 強制検証
