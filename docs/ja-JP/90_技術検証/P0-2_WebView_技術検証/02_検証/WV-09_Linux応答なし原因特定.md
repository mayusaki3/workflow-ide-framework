<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260613-010000Z-K3M2
lang: ja-JP
canonical_title: WV-09 Linux応答なし原因特定
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-09 Linux応答なし原因特定

# WV-09 Linux応答なし原因特定

## 目的

WV-07で発生した Linux 環境における応答なし現象について、WV-08で除外できなかった要素を段階的に切り分け、主因候補を絞り込む。

## 前提

WV-07では GTK Host Window 表示中のマウス移動により応答なしが発生した。

WV-08では GTK / WebKitGTK / WebView 操作を段階的に再導入し、以下の要素単独では応答なしを再現しないことを確認した。

- gtk::init()
- gtk::Window::new()
- window.show_all()
- GTK Window保持
- GtkFixed階層構築
- WebKitGTK生成
- build_gtk()
- WebView初期化
- WebView set_bounds
- WebView visibility制御
- GTKイベントポンプ
- Native Surface表示切替

## 検証構成

| 検証番号 | 検証項目 | 状態 |
|---|---|---|
| [WV-09-01](WV-09-01_WV07_WV08差分分析.md) | WV07_WV08差分分析 | 完了 |
| [WV-09-02](WV-09-02_X11親子Window制御検証.md) | X11親子Window制御検証 | 完了 |
| [WV-09-03](WV-09-03_GtkFixed階層検証.md) | GtkFixed階層検証 | 完了 |
| WV-09-04 | WebKitGTK + eframe / winit 共存検証 | 未実施 |

## 検証結果概要

### WV-09-01

WV-07とWV-08の差分を整理し、未評価要素を抽出した。

### WV-09-02

GTK Host Window 単体を Dock 矩形へ追従させた状態では、応答なしは再現しなかった。

### WV-09-03

GtkFixed階層を追加した状態では、応答なしは再現しなかった。

## 現在の判定

主因候補から除外する要素は以下とする。

- GTK Host Window単体
- X11上のHost Window位置・サイズ同期単体
- GtkFixed階層
- Root Fixed
- Child Fixed

## 次工程

WV-09-04として、WebKitGTK + eframe / winit 共存検証を実施する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-09 Linux応答なし原因特定