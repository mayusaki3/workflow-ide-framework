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

| 検証番号        | 項目                    | 期待結果     |
| ----------- | --------------------- | -------- |
| WV-09-05-01 | 起動確認                  | 起動成功     |
| WV-09-05-02 | Native Child Window生成 | 正常生成     |
| WV-09-05-03 | マウス移動                 | 応答停止有無確認 |
| WV-09-05-04 | Dock移動                | 応答停止有無確認 |
| WV-09-05-05 | 長時間動作                 | 応答停止有無確認 |

## 判定基準

応答停止が再現した場合

- Native Child Window を主因候補とする。

応答停止が再現しない場合

- 実運用同期処理を次調査対象とする。

## 検証結果

実施完了。

### WV-09-05-01 起動確認

- 起動成功

### WV-09-05-02 Native Child Window生成

- 正常生成

### WV-09-05-03 マウス移動

- 応答停止非再現

### WV-09-05-04 Dock移動

- 応答停止非再現
- Dock追従正常

### WV-09-05-05 長時間動作

- 応答停止非再現

## 判定

不合格

Native Child Window を再導入した構成では応答停止を再現できなかった。

## 知見

以下の構成を同時に有効化しても応答停止は再現しなかった。

- GTK Host Window
- Native Child Window
- GtkFixed
- WebKitGTK
- eframe / winit
- WebView::set_bounds()
- WebView::set_visible()
- Dock追従処理

Native Child Window 単体は主因候補から除外可能と判断する。

## 結論

Native Child Window再導入では応答停止を再現できなかった。

Native Child Window管理は主因候補から除外する。

## 次工程

実運用同期処理および Dock追従同期処理を対象とした切り分けを実施する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-05 Native Child Window再導入検証