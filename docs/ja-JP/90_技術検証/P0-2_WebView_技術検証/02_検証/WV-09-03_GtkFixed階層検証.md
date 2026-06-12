目次 > 技術検証 > P0-2_WebView技術検証 > WV-09-03_GtkFixed階層検証

# WV-09-03 GtkFixed階層検証

## 1. 目的

WV-09-02によりGTK Host Window単体では応答なしが再現しないことを確認した。

本検証では以下構成を導入し、GtkFixed階層自体が応答なしの原因となるか確認する。

```text
GTK Window
 └ Root Fixed
     └ Child Fixed
```

WebKitGTKおよびWebViewは使用しない。

## 2. 検証項目

### WV-09-03-01

起動確認

期待結果:

- 起動成功

### WV-09-03-02

Dock追従確認

期待結果:

- Child Fixedを含む構成で追従成功

### WV-09-03-03

マウス移動確認

期待結果:

- 応答なし発生なし

### WV-09-03-04

Native Surface表示切替

期待結果:

- ON/OFF成功

## 3. 判定

応答なし発生:

- GtkFixed階層が原因候補

応答なし未発生:

- GtkFixed階層は原因から除外

---

目次 > 技術検証 > P0-2_WebView技術検証 > WV-09-03_GtkFixed階層検証
