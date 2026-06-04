<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-06 GTK Host Window最小構成
document_type: verification
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-06 GTK Host Window最小構成

# WV-06 GTK Host Window最小構成

## 目的

WV-05で確認された GTK Host Window 独立トップレベル構成問題について調査する。

本検証では、`build_gtk()` が要求する最小 GTK Widget 構成を特定する。

## 背景

WV-05では以下を確認した。

* build_gtk() 成立
* WebView表示成立
* eframe EventLoop成立
* GTK MainContext成立

一方で、

* GTK Host Window が Ubuntu から「応答なし」と判定される

ことを確認した。

イベントループ統合問題ではなく、GTK Host Window 構成問題である可能性が高い。

## 検証項目

### WV-06-01 GTK Window非表示構成

目的

* GTK Window が表示必須か確認する。

実施内容

* show_all() を実行しない。

判定

成功

* WebView表示継続

### WV-06-02 GTK Widget最小構成

目的

* build_gtk() が要求する Widget を確認する。

実施内容

* gtk::Window
* gtk::Fixed

を段階的に削減する。

判定

成功

* WebView生成継続

### WV-06-03 Host Window所有関係調査

目的

* GTK Host Window の所有関係を確認する。

実施内容

* GTK Window の親子関係調査

判定

成功

* Linux実装方針を決定可能

## 評価基準

成功

* GTK Host Window を不要化できる

条件付き成功

* 最小構成を特定できる

失敗

* 構成要件を特定できない

## 結果

未実施

## 結論

未実施

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-06 GTK Host Window最小構成
