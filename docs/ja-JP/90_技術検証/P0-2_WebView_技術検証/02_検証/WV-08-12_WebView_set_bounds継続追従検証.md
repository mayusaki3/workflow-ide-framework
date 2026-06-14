<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000012Z-W812
lang: ja-JP
canonical_title: WV-08-12 WebView set_bounds継続追従検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-12 WebView set_bounds継続追従検証

# WV-08-12 WebView set_bounds継続追従検証

## 目的

Dock矩形変化に追従して WebView::set_bounds() を継続実行した場合に、応答なしが発生するか確認する。

---

## 実施内容

* WebKitGTK WebView を build_gtk() で生成する。
* WebView を static に保持する。
* Dock矩形が変化した場合のみ WebView::set_bounds() を実行する。
* set_visible() は実行しない。
* 継続GTKイベントポンプは実行しない。
* Native Surface表示切替は実行しない。

---

## 結果

* build_gtk() 成功
* 初期 set_bounds() 成功
* Dock矩形変化時の set_bounds() 継続実行成功
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

---

## 判定

成功

---

## 知見

以下は主因ではない可能性が高い。

* WebView::set_bounds() 継続実行
* Dock矩形変化に伴う WebView再配置
* GtkFixed配下での WebView size_allocate 継続実行

---

## 結論

WebView::set_bounds() を Dock矩形変化に追従して継続実行しても、応答なしは再現しなかった。

継続的な WebView再配置処理は主因ではない可能性が高い。

---

次工程:

* WV-08-13 WebView visibility追従検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-12 WebView set_bounds継続追従検証
