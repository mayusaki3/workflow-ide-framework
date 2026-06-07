<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260606-000000Z-BCAY
parent_doc_id: 検証目次
lang: ja-JP
canonical_title: WV-08 GTK完全無効化検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-08 GTK完全無効化検証

# WV-08 GTK完全無効化検証

## 目的

WV-07で確認された応答なし現象が、GTK統合層に起因するか、Hyper-V Ubuntu + eframe / winit 側に起因するかを切り分ける。

WV-08では GTK を完全に無効化し、PoC-2e / egui_dock / eframe のみで応答なしが発生するか確認する。

## 背景

WV-07では以下を確認した。

* Host Window の位置・サイズ同期は成立した。
* WebView / WebKitGTK / wry を除去しても応答なしが発生した。
* Dummy GTK Widget を除去しても応答なしが発生した。
* Child Widget を除去しても応答なしが発生した。
* 実行中の `move_()` / `resize()` を停止しても応答なしが発生した。
* GTKイベントポンプを停止すると GTK Host Window 自体が表示されなかった。

このため、WV-08では GTK を完全に呼び出さない構成へ変更し、PoC-2e / egui_dock / eframe 側だけで問題が再現するかを確認する。

## 前提条件

本検証では、以下の状態を前提とする。

* `gtk::init()` を呼び出さない。
* GTK Host Window を生成しない。
* WebView を生成しない。
* Dummy GTK Widget を生成しない。
* GTKイベントポンプを実行しない。
* PoC-2e / egui_dock / eframe の画面表示と Dock 操作のみを確認する。

## 検証項目

### WV-08-01 GTK完全無効化

目的

* GTK処理を完全に無効化した状態で、PoC-2e / egui_dock / eframe のみが正常動作するか確認する。

実施内容

* `initialize_root_window()` を空処理化する。
* `ensure_webview_initialized()` を空処理化する。
* `sync_child_window()` を空処理化する。
* PoC-2e の Dock 表示、Dock移動、Dockサイズ変更を確認する。
* マウス操作時に応答なしが発生するか確認する。

確認ログ

```text
WV-08-01 GTK disabled
WV-08-01 ensure_webview_initialized skipped
```

判定

* PoC-2e のみで応答なしが発生しない場合、GTK統合層が応答なしの主因候補となる。
* PoC-2e のみで応答なしが発生する場合、Hyper-V Ubuntu + eframe / winit 側が主因候補となる。

結果

* `WV-08-01 GTK disabled` が出力された。
* `WV-08-01 ensure_webview_initialized skipped` が継続出力された。
* GTK Host Window は生成されなかった。
* PoC-2e / egui_dock は表示された。
* DockRect / WebViewTabBarCandidate のログは継続出力された。
* 提示された実行ログ上では応答なしは確認されていない。

判定結果

* GTKを完全に無効化した状態では、応答なしは再現していない。
* 応答なしの主因は、GTK統合層または GTK Toplevel Window と eframe / winit の共存条件である可能性が高い。

評価基準

成功

* GTK完全無効化状態で PoC-2e / egui_dock / eframe が正常動作し、応答なしが発生しない。

条件付き成功

* 応答なしは発生しないが、GTK無効化に伴う未使用コード警告が残る。

失敗

* GTKを無効化しても応答なしが発生する。

結果

条件付き成功

* GTK完全無効化状態で、PoC-2e / egui_dock / eframe の動作は継続した。
* GTK無効化により、未使用 import / static / const / function の警告が発生した。
* 実行ログ上、応答なしは確認されていない。

---

### WV-08-02 GTK Host Window生成検証

目的

GTK Toplevel Window の生成だけで応答なしが発生するか確認する。

WV-08-01 により GTK 完全無効化状態では応答なしは再現していない。

そのため、GTK統合層のどの段階で問題が発生するかを切り分ける。

実施内容

以下のみ実施する。

* `gtk::init()`
* `gtk::Window::new()`

以下は実施しない。

* `window.show_all()`
* WebView生成
* Dummy GTK Widget生成
* Child Widget生成
* GTKイベントポンプ
* GTKイベントflush

想定コード

```rust
match gtk::init() {
    Ok(_) => {
        println!("WV-08-02 gtk::init success");
    }
    Err(err) => {
        println!("WV-08-02 gtk::init failed: {}", err);
        return;
    }
}

let _window = gtk::Window::new(gtk::WindowType::Popup);

println!("WV-08-02 gtk::Window created");
```

確認項目

#### WV-08-02-01

確認内容

* `gtk::init()` 成功確認

期待結果

* `WV-08-02 gtk::init success`

#### WV-08-02-02

確認内容

* `gtk::Window::new()` 成功確認

期待結果

* `WV-08-02 gtk::Window created`

#### WV-08-02-03

確認内容

* eframe継続動作確認

期待結果

* Dock操作可能
* マウス移動可能
* 応答なし発生なし

判定

成功

* 応答なし発生なし

失敗

* 応答なし発生
* ウィンドウ生成失敗

次工程

成功時

* WV-08-03 Window表示検証

失敗時

* GTK生成段階が主因候補

結果:

gtk::init() 成功
gtk::Window::new() 成功

GTK Window表示なし
Dock操作可能
マウス移動可能
応答なし発生なし

判定

* WV-08-02 成功

---

### WV-08-03 Window表示検証

目的

GTK Window の表示処理で応答なしが発生するか確認する。

WV-08-02 により、`gtk::init()` と `gtk::Window::new()` のみでは応答なしが発生しないことを確認した。

そのため、次に `window.show_all()` を追加し、GTK Toplevel Window の表示処理が応答なしの発生条件になるかを切り分ける。

実施内容

以下を実施する。

* `gtk::init()`
* `gtk::Window::new()`
* `window.show_all()`

以下は実施しない。

* GTK Window保持
* GTKイベントflush
* GTKイベントポンプ
* WebView生成
* Dummy GTK Widget生成
* Child Widget生成

確認項目

#### WV-08-03-01

確認内容

* `gtk::init()` 成功確認

期待結果

* `WV-08-03 gtk::init success`

#### WV-08-03-02

確認内容

* `gtk::Window::new()` 成功確認

期待結果

* `WV-08-03 gtk::Window created`

#### WV-08-03-03

確認内容

* `window.show_all()` 成功確認

期待結果

* `WV-08-03 window.show_all done`

#### WV-08-03-04

確認内容

* eframe継続動作確認

期待結果

* Dock操作可能
* マウス移動可能
* 応答なし発生なし

判定

成功

* 応答なし発生なし

失敗

* 応答なし発生
* Window表示失敗

次工程

成功時

* WV-08-04 GTK Window保持検証

失敗時

* GTK Window表示段階が主因候補

---

### WV-08-04 GTK Window保持検証

目的

GTK Window のライフタイム保持が応答なしの発生条件になるか確認する。

WV-08-03 により `window.show_all()` 単体で問題が発生しない場合、次に GTK Window を static 保持した状態で確認する。

実施内容

以下を実施する。

* `gtk::init()`
* `gtk::Window::new()`
* `window.show_all()`
* `GTK_WINDOW = Some(window)`

以下は実施しない。

* GTKイベントflush
* GTKイベントポンプ
* WebView生成
* Dummy GTK Widget生成

確認項目

#### WV-08-04-01

確認内容

* GTK Window保持成功確認

期待結果

* `WV-08-04 GTK_WINDOW stored`

#### WV-08-04-02

確認内容

* eframe継続動作確認

期待結果

* Dock操作可能
* マウス移動可能
* 応答なし発生なし

判定

成功

* 応答なし発生なし

失敗

* 応答なし発生

次工程

成功時

* WV-08-05 GTKイベントflush検証

失敗時

* GTK Window保持段階が主因候補

---

### WV-08-05 GTKイベントflush検証

目的

GTKイベント処理が応答なしの発生条件になるか確認する。

WV-08-04 により GTK Window保持単体で問題が発生しない場合、次に show_all() 直後の GTKイベントflush を追加して確認する。

実施内容

以下を実施する。

* `gtk::init()`
* `gtk::Window::new()`
* `window.show_all()`
* `GTK_WINDOW = Some(window)`
* `flush_gtk_events_bounded()`

以下は実施しない。

* 継続的GTKイベントポンプ
* WebView生成
* Dummy GTK Widget生成

確認項目

#### WV-08-05-01

確認内容

* GTKイベントflush成功確認

期待結果

* `GTK event flush completed`

#### WV-08-05-02

確認内容

* eframe継続動作確認

期待結果

* Dock操作可能
* マウス移動可能
* 応答なし発生なし

判定

成功

* 応答なし発生なし

失敗

* 応答なし発生

次工程

成功時

* WV-09以降の別要因調査

失敗時

* GTKイベントflush段階が主因候補

---

## 得られた知見

WV-08-01

GTK完全無効化
→ 応答なしなし

WV-08-02

gtk::init()
gtk::Window::new()
→ 応答なしなし

---

## 結論

WV-08-01により、応答なし現象は PoC-2e / egui_dock / eframe 単体では再現しない可能性が高い。

WV-07の結果と合わせると、応答なしの主因は GTK Toplevel Window を eframe / winit アプリケーションと同時に動作させる統合層にある可能性が高い。

次段階では、WV-08-03として `window.show_all()` を追加し、GTK Toplevel Window の表示処理が応答なしの発生条件になるかを確認する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-08 GTK完全無効化検証
