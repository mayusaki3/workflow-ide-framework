<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000094Z-W904
lang: ja-JP
canonical_title: WV-09-04 WebKitGTK eframe winit共存検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-04 WebKitGTK eframe winit共存検証

# WV-09-04 WebKitGTK eframe winit共存検証

## 目的

WV-09-03までで除外できなかった要素のうち、WebKitGTK と eframe / winit の共存が Linux 応答停止現象の主因となるか確認する。

## 背景

WV-08では WebKitGTK生成単体では応答停止を再現しなかった。

WV-09-02およびWV-09-03では、X11親子Window制御およびGtkFixed階層でも応答停止を再現しなかった。

そのため、WebKitGTKと eframe / winit の共存状態を確認する。

## 検証構成

* GDK_BACKEND=x11
* eframe
* egui
* egui_dock
* GTK Host Window
* GtkFixed
* WebKitGTK
* WebView
* wry build_gtk()
* Dock矩形追従
* WebView::set_bounds()
* WebView::set_visible(true)

## 検証項目

| 検証番号        | 項目            | 期待結果    | 結果 |
| ----------- | ------------- | ------- | --- |
| WV-09-04-01 | 起動確認          | 起動成功    | 成功 |
| WV-09-04-02 | WebKitGTK生成確認 | 正常生成    | 成功 |
| WV-09-04-03 | マウス移動確認       | 応答停止未発生 | 成功 |
| WV-09-04-04 | 表示切替確認        | 正常追従    | 成功 |
| WV-09-04-05 | 長時間動作確認       | 応答停止未発生 | 成功 |

## 判定基準

応答停止が再現した場合

* WebKitGTK + eframe / winit 共存を主因候補とする

応答停止が再現しない場合

* 実運用同期処理
* Dock追従同期処理

を次調査対象とする。

## 検証結果

成功。

`GDK_BACKEND=x11 cargo run` により起動し、以下を確認した。

* ビルド成功
* WV-09-04 WebView build_gtk success を確認
* WebView::set_bounds() 成功
* WebView::set_visible(true) 成功
* Dock矩形追従成功
* マウス操作中に応答停止未発生

補足:

* `static mut` 参照に関する Rust 2024 互換警告は発生したが、ビルドは成功した。
* libEGL / Mesa / DRI3 関連の警告は発生したが、WebKitGTK WebView生成および操作継続は成功した。

## 判定

成功。

WebKitGTK + eframe / winit 共存状態では、Linux 応答停止現象は再現しなかった。

## 知見

以下の組み合わせでは応答停止を再現しなかった。

* GTK Host Window
* GtkFixed
* WebKitGTK WebView
* eframe / winit
* WebView::set_bounds()
* WebView::set_visible(true)
* GTKイベントflush
* Dock矩形追従

この結果により、WebKitGTK + eframe / winit 共存そのものは主因ではない可能性が高い。

## 結論

WV-09-04では応答停止を再現しなかった。

次調査対象は、WV-07で応答停止が発生した実運用寄りの同期処理との差分に移行する。

## 次工程

WV-09-05 として、WV-07構成との差分を再確認し、実運用同期処理またはDock追従同期処理を対象に追加検証を計画する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-04 WebKitGTK eframe winit共存検証
