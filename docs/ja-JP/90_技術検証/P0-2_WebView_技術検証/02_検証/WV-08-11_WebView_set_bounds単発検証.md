[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-11 WebView set_bounds単発検証

# WV-08-11 WebView set_bounds単発検証

## 目的

WebKitGTK生成後の WebView::set_bounds() 単発実行のみで応答なしが発生するか確認する。

---

## 実施内容

```rust
webview.set_bounds(
    Rect {
        position: (20, 20),
        size: (280, 180),
    }
);
```

実行回数:

* 1回のみ

実施しない内容:

* Dock追従
* 継続同期
* show/hide
* 継続GTKイベントポンプ

---

## 結果

* webview build_gtk success
* webview set_bounds success
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

* WebView set_bounds 単発実行
* GtkFixed配下での size_allocate
* WebView再配置単発処理

---

## 結論

WebKitGTK生成後の単発再配置では応答なしは再現しなかった。

問題は継続的な再配置処理、表示制御、または GTKイベントポンプに存在する可能性が高い。

---

次工程:

* WV-08-12 WebView set_bounds継続追従検証

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-11 WebView set_bounds単発検証
