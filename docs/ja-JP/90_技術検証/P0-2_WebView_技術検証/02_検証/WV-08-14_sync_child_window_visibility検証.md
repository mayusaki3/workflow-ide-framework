<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000014Z-W814
lang: ja-JP
canonical_title: WV-08-14 sync_child_window visibility検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-14 sync_child_window visibility検証

# WV-08-14 sync_child_window visibility検証

## 目的

sync_child_window() 経由で should_show_native_surface に応じて WebView::set_visible(true / false) を実行した場合に、応答なしが発生するか確認する。

---

## 実施内容

* WebKitGTK WebView を build_gtk() で生成する。
* WebView を static に保持する。
* Dock矩形が変化した場合のみ WebView::set_bounds() を実行する。
* ensure_webview_initialized() 側の set_visible(true) は維持する。
* sync_child_window() で should_show_native_surface に応じて WebView::set_visible(true / false) を実行する。
* 継続GTKイベントポンプは実行しない。

---

## 結果

* build_gtk() 成功
* 初期 set_bounds() 成功
* Dock矩形変化時の set_bounds() 継続実行成功
* sync_child_window() 経由の set_visible(true) 成功
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

補足:

* WV-08-14 set_visible(true) は大量に実行された。
* WV-08-14 set_visible(false) は今回の操作では未実行であった。

---

## 判定

部分成功

---

## 知見

以下は主因ではない可能性が高い。

* sync_child_window() 経由の WebView::set_visible(true)
* WebView::set_bounds() 継続実行と sync_child_window() 経由 set_visible(true) の組み合わせ

未評価:

* sync_child_window() 経由の WebView::set_visible(false)

---

## 結論

sync_child_window() 経由で WebView::set_visible(true) を大量実行しても、応答なしは再現しなかった。

ただし set_visible(false) 経路は未実行であるため、visibility制御を主因から完全に除外するには、set_visible(false) を強制実行する検証が必要である。

---

次工程:

* WV-08-15 WebView set_visible(false) 強制検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-14 sync_child_window visibility検証
