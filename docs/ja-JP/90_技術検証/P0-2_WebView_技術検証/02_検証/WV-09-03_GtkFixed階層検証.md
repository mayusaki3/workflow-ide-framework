<!--
HLDocS:LLM-MANAGED
lang: ja-JP
doc_id: workflow-ide-framework-p0-2-webview-wv-09-03
canonical_title: WV-09-03 GtkFixed階層検証
document_type: technical_validation
canonical_document: true
status: completed
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-03 GtkFixed階層検証

# WV-09-03 GtkFixed階層検証

## 目的

WV-09-02によりGTK Host Window単体では応答なしが再現しないことを確認した。

本検証ではGtkFixed階層を導入し、階層構造自体が応答なしの原因となるか確認する。

## 検証構成

GTK Window
 └ Root Fixed
     └ Child Fixed

WebKitGTKおよびWebViewは使用しない。

## 検証結果

### WV-09-03-01 起動確認

結果: 成功

### WV-09-03-02 Dock追従確認

結果: 成功

GTK Host WindowはDock矩形へ追従した。

### WV-09-03-03 マウス移動確認

結果: 応答なし再現なし

### WV-09-03-04 Native Surface表示切替

結果: 成功

Debug: Show Native Surface OFF 時にタブ切替へ追従して表示ON/OFFされた。

## 判定

GtkFixed階層は応答なしの主因候補から除外する。

除外対象:

- Root Fixed
- Child Fixed
- GtkFixed階層

## 次工程

WV-09-04 WebKitGTK + eframe / winit 共存検証を実施する。

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-03 GtkFixed階層検証
