<!--
HLDocS:LLM-MANAGED
lang: ja-JP
doc_id: workflow-ide-framework-p0-2-webview-wv-09-02
canonical_title: WV-09-02 X11親子Window制御検証
document_type: technical_validation
canonical_document: true
status: completed
-->

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-02 X11親子Window制御検証

# WV-09-02 X11親子Window制御検証

## 目的

GTK Host Window単体をDock矩形へ追従させ、X11上のHost Window位置・サイズ同期が応答なしの主因となるか確認する。

## 検証構成

- GDK_BACKEND=x11
- GTK Host Windowのみ生成
- WebKitGTKなし
- wry WebViewなし
- GtkFixed配下WebViewなし
- Dock矩形に合わせてmove / resizeを実行

## 検証項目

| 検証番号 | 項目 | 期待結果 |
|---|---|---|
| WV-09-02-01 | 起動確認 | 起動成功 |
| WV-09-02-02 | Dock追従確認 | Host WindowがDock矩形へ追従する |
| WV-09-02-03 | マウス移動確認 | 応答なしが発生しない |
| WV-09-02-04 | 表示切替確認 | Native Surface表示切替に追従する |

## 検証結果

| 検証番号 | 結果 |
|---|---|
| WV-09-02-01 | 成功 |
| WV-09-02-02 | 成功 |
| WV-09-02-03 | 応答なし再現なし |
| WV-09-02-04 | 成功 |

## 判定

GTK Host Window単体およびX11上のHost Window同期処理単体は、応答なしの主因候補から除外する。

## 次工程

WV-09-03 GtkFixed階層検証を実施する。

---

[目次](../../目次.md) > [技術検証目次](../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > [WV-09 Linux応答なし原因特定](WV-09_Linux応答なし原因特定.md) > WV-09-02 X11親子Window制御検証
