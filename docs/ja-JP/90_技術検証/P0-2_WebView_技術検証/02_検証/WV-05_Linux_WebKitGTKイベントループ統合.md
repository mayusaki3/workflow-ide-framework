<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-05 Linux WebKitGTKイベントループ統合
document_type: verification
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-05 Linux WebKitGTKイベントループ統合

# WV-05 Linux WebKitGTKイベントループ統合

## 目的

WV-04で選定した Linux標準方式について、WebKitGTK と eframe のイベントループ共存可否を確認する。

選定済み方式

* wry build_gtk()
* gtk::Fixed
* gtk::Fixed::move_()
* gtk::Fixed::set_size_request()

本検証では、WebView生成後に発生する応答停止問題の原因を特定し、Linux版 Framework 実装方式を確定する。

## 背景

WV-04では以下を確認した。

* Wayland環境で WebView生成成功
* URL表示成功
* Child Surface移動成功
* Child Surfaceリサイズ成功
* Windows版と同等の Surface 管理I/F 実現可能

一方で、WebView生成後に以下の問題が発生した。

* eframe UI 応答停止
* GTK Host Window 応答停止

配置方式の問題ではないことを確認済み。

確認済み事項

* build_as_child() 非採用
* set_bounds() 非依存
* move_() 非依存
* set_size_request() 非依存
* 毎フレーム同期 非依存
* flush_gtk_events() 非依存

## 検証環境

| 項目             | 内容              |
| -------------- | --------------- |
| OS             | Linux (Wayland) |
| GUI Framework  | egui            |
| Window Backend | eframe          |
| Dock System    | egui_dock       |
| WebView        | wry             |
| GTK            | GTK3            |
| Web Engine     | WebKitGTK       |

## 仮説

### 仮説1

GTK MainContext所有スレッドと WebView生成スレッドが一致していない。

### 仮説2

WebKitGTKが要求する MainContext が適切に駆動されていない。

### 仮説3

winit EventLoop と GTK MainLoop が競合している。

## 検証項目

### WV-05-01 MainContext所有状態確認

#### 目的

WebView生成時の MainContext 所有状態を確認する。

#### 実施内容

以下を記録する。

* glib::MainContext::default().is_owner()

#### 判定

成功

* 所有状態を取得できる

### WV-05-02 GTKイベント継続確認

#### 目的

GTKイベントループ継続状態を確認する。

#### 実施内容

以下を登録する。

* glib::timeout_add_local()

定期ログ出力を行う。

#### 判定

成功

* GTK alive が継続出力される

### WV-05-03 eframeイベント継続確認

#### 目的

eframeイベントループ継続状態を確認する。

#### 実施内容

以下を記録する。

* sync_child_window() 呼び出しログ

#### 判定

成功

* eframe alive が継続出力される

### WV-05-04 応答停止箇所特定

#### 目的

停止対象を分類する。

#### 分類

* GTK停止
* eframe停止
* 両方停止
* WebKit Process停止

#### 判定

成功

* 停止対象を特定できる

## 評価基準

### 成功

* eframe と WebKitGTK が同時応答
* WebView表示継続
* Child Surface移動継続
* Child Surfaceリサイズ継続

### 条件付き成功

* 応答停止原因を特定
* Linux実装方式を決定可能

### 失敗

* 原因特定不能

## 結果

完了

### WV-05-01 MainContext所有状態確認

結果

* 成功

確認内容

* WebView生成前後で `glib::MainContext::default().is_owner()` が `true` であることを確認した。

判定

* GTK MainContext所有スレッド不一致は主因ではない。

### WV-05-02 GTKイベント継続確認

結果

* 成功

確認内容

* `glib::timeout_add_local()` による `GTK timer alive` が継続出力されることを確認した。

判定

* GTK MainContext は継続駆動できている。

### WV-05-03 eframeイベント継続確認

結果

* 成功

確認内容

* `sync_child_window()` 経由で `eframe alive` が継続出力されることを確認した。

判定

* eframe / winit EventLoop は継続動作している。

### WV-05-04 応答停止箇所特定

結果

* 条件付き成功

確認内容

* `build_gtk()` は成功する。
* WebView表示は成功する。
* eframe alive は継続する。
* GTK timer alive は継続する。
* ただし GTK Host Window には Ubuntu 側で「応答がありません」と表示される。

判定

* プロセス停止ではない。
* eframe停止ではない。
* GTK MainContext停止ではない。
* WebKitGTK停止ではない。
* 残課題は、GTK独立トップレベルウィンドウと eframeトップレベルウィンドウの共存構成にある可能性が高い。

## 結論

WV-05 Linux WebKitGTKイベントループ統合は、条件付き成功とする。

確認できた事項

* `build_gtk()` による WebView生成は成立する。
* WebView表示は成立する。
* eframe / winit EventLoop は継続動作する。
* GTK MainContext は継続駆動できる。
* 上限付き GTKイベント処理により、eframe と GTK の双方を停止させずに動作させられる。

判定

* 応答停止問題の主因は、イベントループ統合そのものではない。
* 主因候補は、GTK Host Window を独立トップレベルとして生成している構成である。

次工程

* GTK Host Window が必須か確認する。
* build_gtk() が要求する最小 GTK Widget 構成を確認する。
* GTK Window を生成しない構成が可能か確認する。
* GTK Host Window の所有関係を調査する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-05 Linux WebKitGTKイベントループ統合
