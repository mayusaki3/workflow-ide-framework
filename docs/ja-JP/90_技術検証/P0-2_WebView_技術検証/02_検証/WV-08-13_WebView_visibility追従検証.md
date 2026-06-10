[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-13 WebView visibility追従検証

# WV-08-13 WebView visibility追従検証

## 目的

WebView::set_visible(true) を Dock追従処理内で実行した場合に、応答なしが発生するか確認する。

---

## 実施内容

* WebKitGTK WebView を build_gtk() で生成する。
* WebView を static に保持する。
* Dock矩形が変化した場合のみ WebView::set_bounds() を実行する。
* WebView配置矩形が存在する場合に WebView::set_visible(true) を実行する。
* WebView配置矩形が存在しない場合に WebView::set_visible(false) を実行する実装を追加する。
* 継続GTKイベントポンプは実行しない。
* sync_child_window() 経由の visibility 制御は実行しない。

---

## 結果

* build_gtk() 成功
* 初期 set_bounds() 成功
* Dock矩形変化時の set_bounds() 継続実行成功
* set_visible(true) 成功
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

補足:

* Hide Native Surface ログは発生した。
* ただし WV-08-13 set_visible(false) は未実行であった。

---

## 判定

部分成功

---

## 知見

以下は主因ではない可能性が高い。

* WebView::set_visible(true)
* WebView::set_bounds() 継続実行と set_visible(true) の組み合わせ

未評価:

* WebView::set_visible(false)

---

## 結論

WebView::set_visible(true) を Dock追従中に繰り返し実行しても、応答なしは再現しなかった。

ただし、set_visible(false) 経路はこの検証では実行されなかったため、別検証で確認する必要がある。

---

次工程:

* WV-08-14 sync_child_window visibility検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-13 WebView visibility追従検証
