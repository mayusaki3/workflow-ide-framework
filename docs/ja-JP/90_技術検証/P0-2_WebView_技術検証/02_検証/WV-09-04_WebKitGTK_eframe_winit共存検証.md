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

WV-09-03までで除外できなかった要素のうち、WebKitGTK と eframe / winit の共存が Linux 応答なし現象の主因となるか確認する。

## 背景

WV-08では WebKitGTK生成単体では応答なしを再現しなかった。

WV-09-02およびWV-09-03では、X11親子Window制御およびGtkFixed階層でも応答なしを再現しなかった。

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

## 検証項目

| 検証番号        | 項目            | 期待結果    |
| ----------- | ------------- | ------- |
| WV-09-04-01 | 起動確認          | 起動成功    |
| WV-09-04-02 | WebKitGTK生成確認 | 正常生成    |
| WV-09-04-03 | マウス移動確認       | 応答なし未発生 |
| WV-09-04-04 | 表示切替確認        | 正常追従    |
| WV-09-04-05 | 長時間動作確認       | 応答なし未発生 |

## 判定基準

応答なしが再現した場合

* WebKitGTK + eframe / winit 共存を主因候補とする

応答なしが再現しない場合

* 実運用同期処理
* Dock追従同期処理

を次調査対象とする。

## 検証結果

未実施

## 判定

未実施

## 次工程

結果に応じて WV-09-05 を計画する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-04 WebKitGTK eframe winit共存検証
