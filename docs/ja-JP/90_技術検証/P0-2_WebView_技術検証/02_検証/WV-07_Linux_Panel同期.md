<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260614-000008Z-WV07
lang: ja-JP
canonical_title: WV-07 Linux Panel同期
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-07 Linux Panel同期

# WV-07 Linux Panel同期

## 目的

WV-06で確認した Linux版 WebView 実装方式を前提として、Windows版と同等の方式で Dock Panel と GTK Host Window を同期できることを確認する。

当初は以下を前提条件として、Linux環境で Panel 同期方式の成立性を確認する想定だった。

```bash
GDK_BACKEND=x11 cargo run
```

## 背景

WV-06では以下を確認した。

* build_gtk() による WebView生成成功
* GTK MainContext 動作継続
* GTKイベントポンプ動作継続
* Wayland環境で応答なし再現
* X11環境で応答なし非再現の可能性

ただし、WV-07で検証を進めた結果、応答なしは WebView / WebKitGTK / wry 固有ではなく、GTK Toplevel Window と eframe / winit の共存条件に関連する可能性が高いことが分かった。

## 検証項目

### WV-07-01 Host Window位置・サイズ同期

目的

* GTK Host Window 自体を Dock Panel の位置・サイズに同期できることを確認する。

実施内容

* Child Surface ではなく GTK Host Window 自体へ `move_()` / `resize()` を適用した。
* WebView は wry `WebViewBuilder::build_gtk()` により生成した。
* Dock Panel の位置変更およびサイズ変更に追従するか確認した。

結果

* Host Window は Dock Panel の移動量に追従した。
* Host Window は Dock Panel のサイズ変更に追従した。
* ただし、座標は Dock 基準ではなく desktop 左上起点として扱われた。
* WebView は操作不能だった。
* Host Window 上でマウス移動すると応答なしが発生した。

判定

* 位置・サイズ同期機構は成立。
* WebView操作性および応答なし問題は未解決。

### WV-07-02 Dummy GTK Widget 切り分け

目的

* 応答なしの主因が WebKitGTK / wry WebView 側か、GTK Host Window 側かを切り分ける。

実施内容

* `WebViewBuilder::build_gtk()` を呼び出さない。
* WebView の代わりに `gtk::Label` を配置した。
* GTK Host Window の同期処理は維持した。

結果

* Dummy GTK Widget は表示された。
* Host Window は Dock移動およびサイズ変更に追従した。
* Host Window 上でマウス移動すると応答なしが発生した。

判定

* WebView / WebKitGTK / wry 固有問題ではない可能性が高い。

### WV-07-03 GTKイベントポンプ停止

目的

* `gtk::main_iteration_do(false)` を含む GTKイベントポンプが応答なし発生条件か確認する。

実施内容

* Dummy GTK Widget 構成を維持した。
* GTKイベントポンプを停止した。

結果

* Dummy GTK Widget は表示されなかった。
* 応答なしは発生しなかった。

判定

* GTKイベントポンプを停止すると GTK Window / Widget の表示処理が成立しない。
* この結果のみでは GTKイベントポンプが応答なし主因とは断定できない。

### WV-07-04 GTKイベントポンプ500ms制限

目的

* GTKイベントポンプ頻度が応答なしに影響するか確認する。

実施内容

* Dummy GTK Widget 構成を維持した。
* GTKイベントポンプ間隔を 500ms に制限した。

結果

* Dummy GTK Widget は表示された。
* Host Window は Dock移動およびサイズ変更に追従した。
* Host Window 上でマウス移動すると応答なしが発生した。

判定

* GTKイベントポンプ頻度だけが主因ではない。

### WV-07-05 実行中 move / resize 停止

目的

* 実行中の `window.move_()` / `window.resize()` が応答なし発生条件か確認する。

実施内容

* Dummy GTK Widget 構成を維持した。
* 初期配置時のみ `window.move_()` / `window.resize()` を実行した。
* `sync_child_window()` 内の実行中 `move_()` / `resize()` は停止した。

結果

* Dummy GTK Widget は表示された。
* 初期位置に Host Window が表示された。
* Host Window 上でマウス移動すると応答なしが発生した。

判定

* 実行中の `move_()` / `resize()` は主因ではない。

### WV-07-06 Host Window のみ

目的

* Child Widget / Dummy Label / gtk::Fixed 配下の要素が応答なし発生条件か確認する。

実施内容

* GTK Host Window のみを生成した。
* Child Widget / Dummy Label を生成しない構成とした。
* GTKイベントポンプは有効のままとした。

結果

* GTK Host Window は表示された。
* Child Widget / Dummy Label は表示されなかった。
* Host Window 上でマウス移動すると応答なしが発生した。

判定

* WebView / Dummy Widget / Child Widget / gtk::Fixed 配下要素は主因ではない。
* GTK Toplevel Window が画面表示された状態で応答なしが発生する。

### WV-07-07 Host Window のみ + GTKイベントポンプ停止

目的

* GTK Host Window のみ構成で、GTKイベントポンプを完全停止した場合の挙動を確認する。

実施内容

* GTK Host Window のみ構成を維持した。
* `gtk::events_pending()` / `gtk::main_iteration_do(false)` を実行しない構成とした。

結果

* GTK Host Window は表示されなかった。
* GTKイベントポンプ停止ログのみ出力された。
* Host Window が表示されないため、Host Window 上でのマウス操作検証は成立しなかった。

判定

* GTK Toplevel Window の表示には GTKイベント処理が必要。
* 応答なしは「GTK Window が実際に画面表示されている状態」で発生する。
* GTKイベントポンプ単独主因とは断定できない。

## 評価基準

成功

* Windows版と同等の Panel同期が成立し、応答なしが発生しない。

条件付き成功

* Host Window の位置・サイズ同期は成立するが、応答なし問題が残る。

失敗

* Linux版 Panel同期方式として実用可能な状態に到達しない。

## 結果

条件付き失敗

* Host Window の位置・サイズ同期は成立した。
* desktop 左上起点ではあるが、Dock移動量およびサイズ変更への追従は確認できた。
* WebView / WebKitGTK / wry を除去しても応答なしが発生した。
* Dummy Widget / Child Widget / 実行中 move / resize を除去しても応答なしが発生した。
* GTK Host Window が画面表示された状態で、Host Window 上のマウス移動により応答なしが発生する。

## 結論

WV-07では、Linux版で Windows版と同等の「別Native Windowを Dock Panel に重ねる方式」を検証した。

Host Window の位置・サイズ同期自体は成立したが、GTK Toplevel Window が画面表示された状態で応答なしが発生するため、Linux版 Panel同期方式としては実用成立しなかった。

応答なしの主因は WebView / WebKitGTK / wry 固有ではなく、GTK Toplevel Window と eframe / winit の共存、または Hyper-V Ubuntu 環境上の X11 / llvmpipe / GTK Window 表示条件にある可能性が高い。

WV-08では、GTK処理を完全に無効化し、PoC-2e / egui_dock / eframe のみで応答なしが発生するかを確認する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-07 Linux Panel同期
