<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000095Z-W905
lang: ja-JP
canonical_title: WV-09-05 Native Child Window再導入検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-05 Native Child Window再導入検証

# WV-09-05 Native Child Window再導入検証

## 目的

WV-07で使用していた Native Child Window 構成を再導入し、Linux応答停止現象が再現するか確認する。

## 背景

WV-09-04では以下の構成で応答停止を再現しなかった。

- GTK Host Window
- GtkFixed
- WebKitGTK
- eframe / winit
- WebView::set_bounds()
- WebView::set_visible()

そのため、残る主因候補として Native Child Window 管理を検証対象とする。

## 検証構成

- GDK_BACKEND=x11
- eframe
- egui
- GTK Host Window
- Native Child Window
- WebKitGTK
- Dock追従処理

## 検証項目

| 検証番号 | 項目 | 期待結果 |
| --- | --- | --- |
| WV-09-05-01 | 起動確認 | 起動成功 |
| WV-09-05-02 | Native Child Window生成 | 正常生成 |
| WV-09-05-03 | マウス移動 | 応答停止有無確認 |
| WV-09-05-04 | Dock移動 | 応答停止有無確認 |
| WV-09-05-05 | 長時間動作 | 応答停止有無確認 |

## 判定基準

応答停止が再現した場合

- Native Child Window を主因候補とする。

応答停止が再現しない場合

- 実運用同期処理を次調査対象とする。

## 検証結果

未実施

## 判定

未実施

## 知見

未実施

## 結論

未実施

## 次工程

検証結果により決定する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-05 Native Child Window再導入検証
