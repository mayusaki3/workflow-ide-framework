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

* 成功
* WebView表示継続

### WV-06-02 GTK Widget最小構成

目的

* build_gtk() が要求する Widget を確認する。

実施内容

* gtk::Window
* gtk::Fixed

を段階的に削減する。

判定

* 成功
* WebView生成継続

### WV-06-03 Host Window所有関係調査

目的

* GTK Host Window の所有関係を確認する。

実施内容

* GTK Window の親子関係調査

判定

* 成功
* Linux実装方針を決定可能

### WV-06-04 Host Window種別変更

目的

* GTK Host Window の Window 種別が応答なし判定に影響するか確認する。

実施内容

* Utility Window
* Popup Window
* 1x1 Window

へ変更して比較。

判定

* 成功

結果

* いずれも応答なし発生。
* Window種別は主因ではない。

### WV-06-05 GTK入力イベント切り分け

目的

* WebViewへの入力イベントが応答なし発生条件か確認する。

実施内容

* child_fixed.set_sensitive(false)
* child_fixed.set_can_focus(false)

を実施。

判定

* 成功

結果

* 応答なし継続発生。
* 入力イベントは主因ではない。

### WV-06-06 Wayland/X11比較

目的

* 実装問題か実行環境問題か切り分ける。

実施内容

Wayland

```bash
cargo run
```

X11

```bash
GDK_BACKEND=x11 cargo run
```

判定

* 成功

結果

* Wayland環境では応答なし発生。
* X11環境では応答なし非再現。
* X11ではHost Windowが左上へ張り付くが動作継続。

観測ログ

Wayland

```bash
MESA: error: ZINK: failed to choose pdev
egl: failed to create dri2 screen
```

X11

```bash
DRI3 error: Could not get DRI3 device
Ensure your X server supports DRI3
```

環境情報

* Hyper-V
* Ubuntu
* XDG_SESSION_TYPE=wayland
* OpenGL renderer = llvmpipe
* Accelerated = no

## 評価基準

成功

* GTK Host Window を不要化できる

条件付き成功

* 最小構成を特定できる

失敗

* 構成要件を特定できない

## 結果

実施完了

確認事項

* build_gtk() による WebView生成成功
* GTK MainContext 動作継続
* GTKイベントポンプ動作継続
* GTK Host Window 非表示構成成立
* Host Window種別変更では改善なし
* 入力イベント無効化では改善なし
* Wayland環境で応答なし再現
* X11環境で応答なし非再現

追加観測

* Hyper-V上のUbuntuでOpenGLアクセラレーションは無効
* llvmpipeによるソフトウェアレンダリングを使用

## 結論


WV-04で選定した

* build_gtk()
* gtk::Fixed
* move_()
* set_size_request()

によるLinux実装方式は成立する。

GTKイベントループ統合問題は確認されなかった。

また、

* Host Window種別
* 入力イベント

は応答なしの主因ではないことを確認した。

一方、

* Wayland環境では応答なし再現
* X11環境では応答なし非再現

を確認した。

そのため、WV-05で確認された応答なし現象は、

* Wayland環境で再現
* X11環境で非再現

することを確認した。

現時点では、

* Wayland固有問題
* Wayland + WebKitGTK の組み合わせ
* Hyper-V上のWayland実装
* llvmpipe利用環境

のいずれかで発生している可能性がある。

原因の特定には、

* 実機Linux
* GPUアクセラレーション有効環境

での追加検証が必要である。

次工程では、

GDK_BACKEND=x11

を前提として、

Windows版と同等の

* Panel位置同期
* Panelサイズ同期
* WebView表示同期

を実装し、Linux版方式の成立性を確認する。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-06 GTK Host Window最小構成
