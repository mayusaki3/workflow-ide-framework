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

(検証後に記載)

---

## 判定

(検証後に記載)

---

## 知見

(検証後に記載)

---

## 結論

(検証後に記載)

---

次工程:

* 検証結果に応じて決定

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-15 WebView set_visible(false) 強制検証
