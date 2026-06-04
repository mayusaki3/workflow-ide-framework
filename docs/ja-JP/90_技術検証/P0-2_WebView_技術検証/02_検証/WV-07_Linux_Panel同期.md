<!--
HLDocS:LLM-MANAGED
doc_id: WV-07_Linux_Panel同期
parent_doc_id: 検証目次
lang: ja-JP
canonical_title: WV-07 Linux Panel同期
document_type: verification
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-07 Linux Panel同期

# WV-07 Linux Panel同期

## 目的

WV-06で確認した Linux版 WebView 実装方式を前提として、Windows版と同等の方式で Dock Panel と GTK Host Window を同期できることを確認する。

本検証では、以下を前提条件とする。

```bash
GDK_BACKEND=x11 cargo run
```

## 背景

WV-06では以下を確認した。

* build_gtk() による WebView生成成功
* GTK MainContext 動作継続
* GTKイベントポンプ動作継続
* Wayland環境で応答なし再現
* X11環境で応答なし非再現

そのため、本検証では Wayland環境での応答なし問題は扱わず、X11バックエンドを前提として Panel同期方式の成立性を確認する。

## 検証項目

### WV-07-01 Host Window位置同期

目的

* Dock Panel の位置変更に Host Window が追従できることを確認する。

実施内容

* Dock Panel位置を取得する。
* Host WindowをDock Panel位置へ移動する。
* Dock移動時に追従することを確認する。

判定

* Host Window が Dock Panel 位置へ追従する。

### WV-07-02 Host Windowサイズ同期

目的

* Dock Panel のサイズ変更に Host Window が追従できることを確認する。

実施内容

* Dock Panelサイズを取得する。
* Host WindowサイズをDock Panelサイズへ同期する。
* ウィンドウサイズ変更時に追従することを確認する。

判定

* Host Window サイズが Dock Panel サイズへ追従する。

### WV-07-03 Dock再配置同期

目的

* Dockレイアウト変更時に Host Window が追従できることを確認する。

実施内容

* 左右分割を実施する。
* 上下分割を実施する。
* タブ化を実施する。
* タブ解除を実施する。

判定

* Host Window が新しい Dock Panel 位置へ移動する。

### WV-07-04 WebView表示制御

目的

* WebViewタブ表示状態と Host Window 表示状態が同期することを確認する。

実施内容

* WebViewタブを表示する。
* 他タブを表示する。
* WebViewタブを再表示する。

判定

* 非表示時に Host Window が隠れる。
* 再表示時に Host Window が復帰する。

### WV-07-05 タブドラッグ同期

目的

* Dockタブドラッグ時の Host Window 表示制御を確認する。

実施内容

* タブドラッグを開始する。
* ドラッグ中の表示状態を確認する。
* ドラッグ終了後の表示状態を確認する。

判定

* ドラッグ中に Host Window が適切に非表示となる。
* ドラッグ終了後に Host Window が復帰する。

### WV-07-06 長時間動作確認

目的

* Panel同期処理が継続動作することを確認する。

実施内容

* 5分以上放置する。
* Dock操作を実施する。
* サイズ変更を実施する。

判定

* 応答なしが発生しない。
* 同期処理が継続する。

## 評価基準

成功

* Windows版と同等の Panel同期が成立する。

条件付き成功

* 基本的な位置同期およびサイズ同期が成立する。

失敗

* Panel同期方式が成立しない。

## 結果

未実施

## 結論

未実施

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-07 Linux Panel同期
