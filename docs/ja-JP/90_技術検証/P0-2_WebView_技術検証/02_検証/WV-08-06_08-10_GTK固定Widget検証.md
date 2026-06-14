<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260606-000002Z-BCAY
lang: ja-JP
canonical_title: WV-08-06_08-10 GTK固定Widget検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-06_08-10 GTK固定Widget検証

# WV-08-06_08-10 GTK固定Widget検証

## 目的

GTK Widget階層の生成・配置・移動・サイズ変更が応答なしの原因かを切り分ける。

WV-08-05までで GTK Window と単発イベント処理は無罪候補となったため、本検証では Fixed ベースのWidget階層を段階的に構築する。

---

## WV-08-06 Root Fixed生成検証

### 目的

Root Fixed生成および GTK Window への attach のみで問題が発生するか確認する。

### 実施内容

```rust
let root_fixed = gtk::Fixed::new();

window.add(&root_fixed);

ROOT_FIXED = Some(root_fixed);
```

### 結果

* root_fixed created
* root_fixed attached
* ROOT_FIXED stored
* GTK event flush completed
* 白Window表示
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

### 判定

成功

### 知見

以下は主因ではない可能性が高い。

* gtk::Fixed::new()
* WindowへのFixed attach
* Root Fixed保持

---

## WV-08-07 Child Fixed生成検証

### 目的

Child Fixed生成および Root Fixed への attach のみで問題が発生するか確認する。

### 実施内容

```rust
let child_fixed = gtk::Fixed::new();

root_fixed.put(&child_fixed, 0, 0);

CHILD_FIXED = Some(child_fixed);
```

### 結果

* child_fixed created
* child_fixed attached
* CHILD_FIXED stored
* GTK event flush completed
* 白Window表示
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

### 判定

成功

### 知見

以下は主因ではない可能性が高い。

* Child Fixed生成
* Root Fixed → Child Fixed階層
* Fixed階層構築

---

## WV-08-08 Child Fixed move/resize検証

### 目的

Child Fixed の move / resize のみで問題が発生するか確認する。

### 実施内容

```rust
root_fixed.move_(&child_fixed, 100, 100);

child_fixed.set_size_request(300, 200);
```

### 結果

* child_fixed moved
* child_fixed resized
* GTK event flush completed
* 白Windowが拡大
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

### 判定

成功

### 知見

以下は主因ではない可能性が高い。

* Fixed move
* Fixed resize
* GTKレイアウト更新

---

## WV-08-09 GTK Label追加検証

### 目的

GTK Label追加のみで応答なしが再現するか確認する。

### 実施内容

```rust
let label = gtk::Label::new(Some("WV-08-09"));

child_fixed.put(&label, 0, 0);
```

### 結果

* label created
* label attached
* GTK event flush completed
* 白Window表示
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

### 判定

成功

### 知見

以下は主因ではない可能性が高い。

* GTK Label生成
* GTK Widget追加
* Fixed配下へのWidget配置

---

## WV-08-10 WebKitGTK最小生成検証

### 目的

WebKitGTK WebView生成および GtkFixed への attach のみで応答なしが再現するか確認する。

### 実施内容

```rust
let webview = WebViewBuilder::new()
    .with_url("about:blank")
    .with_bounds(...)
    .build_gtk(&child_fixed);
```

### 結果

* webview build_gtk success
* WEBVIEW_CREATED=true
* GTK event flush completed
* Dock移動可能
* Dockサイズ変更可能
* マウス操作可能
* 応答なし発生せず

### 判定

成功

### 知見

以下は主因ではない可能性が高い。

* WebKitGTK生成
* build_gtk()
* GtkFixed attach
* WebView初期化

---

## WV-08-06 ～ WV-08-10総括

除外できた要因:

* gtk::Fixed::new()
* Root Fixed attach
* Child Fixed attach
* Fixed階層構築
* Child Fixed move
* Child Fixed resize
* GTKレイアウト更新
* GTK Label生成
* GTK Widget追加
* WebKitGTK生成
* build_gtk()
* GtkFixed attach
* WebView初期化

現在の有力候補:

優先度高:

1. WebView set_bounds継続実行
2. 継続GTKイベントポンプ
3. WebKitGTK + eframe / winit 共存
4. WebView visibility制御

優先度中:

5. Native Surface表示切替
6. Dock追従同期処理

### 現時点の結論

WV-08-10時点では GTK Widget階層および WebKitGTK生成だけでは応答なしは再現していない。

GTK基盤および WebKitGTK初期化経路は正常動作している可能性が高い。

今後の切り分け対象は WebView再配置処理および継続同期処理である。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > [WV-08 GTK完全無効化検証](./WV-08_GTK完全無効化検証.md) > WV-08-06_08-10 GTK固定Widget検証
