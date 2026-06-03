<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-03 Linux egui共存
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-03 Linux egui共存

# WV-03 Linux egui共存

## 目的

Linux 環境において、WV-00 ～ WV-02 で Windows 上で成立した構成が同様に成立することを確認する。

本検証では Linux 上で egui_dock、eframe および wry を組み合わせた WebView Support Panel 構成の成立性を確認する。

本検証の結果を基に macOS 検証へ進む。

## 前提条件

### WV-00

確認済み事項

- Dock矩形取得
- Dock移動検知
- Dockリサイズ検知
- Child Window Overlay方式成立

### WV-01

確認済み事項

- WebView生成成功
- URL表示成功
- build_as_child方式成立

### WV-02

確認済み事項

- egui共存成立
- Dock移動成功
- Dockリサイズ成功
- Dockタブ切替成功
- Floating禁止方式成立
- WebView Panel へのタブドラッグ時のみ Hide 成功
- アクティブタブ判定成功
- Child Window 初期配置方式成立

### 対象環境

OS

- Ubuntu LTS
- Fedora
- その他主要Linuxディストリビューション

対象ライブラリ

- eframe
- egui
- egui_dock
- wry

## PoC

### 検証内容

#### WV-03-01 Linuxビルド

確認項目

- cargo check
- cargo build
- 依存解決

#### WV-03-02 Linux起動

確認項目

- アプリ起動
- Dock表示
- 異常終了有無

#### WV-03-03 Dock共存

確認項目

- Dock移動
- Dockリサイズ
- Dockタブ切替

#### WV-03-04 WebView表示

確認項目

- WebView生成
- URL表示
- Child Window表示
- Child Window初期配置（起動時非表示→表示）

#### WV-03-05 Child Window追従

確認項目

- Dock移動追従
- Dockリサイズ追従
- タブ切替追従
- アクティブタブ切替
- タブドラッグ中Hide
- 初回表示時フラッシュ有無

#### WV-03-06 Floating禁止方式

確認項目

- allowed_in_windows()
- Floating禁止
- Dock操作への影響
- アクティブタブ制御
- ネイティブサーフェス退避制御

### 実施結果

#### WV-03-01 Linuxビルド

確認結果

* Windows依存コード（windows crate参照）の platform 層移動後、Linux ビルドへ進行可能であることを確認。
* Ubuntu 26.04 LTS 環境において、初回ビルド時に以下の pkg-config 解決エラーを確認。

  * glib-2.0
  * gobject-2.0
  * gio-2.0
  * gdk-3.0
* 原因は Linux 開発パッケージ未導入であり、アプリケーションコードの問題ではなかった。
* 以下パッケージ導入後、依存解決エラーは解消した。

```bash
sudo apt install \
  build-essential \
  pkg-config \
  libglib2.0-dev \
  libgio-2.0-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev
```

判定

* 合格

備考

* Linux 版 wry は WebKitGTK / GTK3 / GLib 開発パッケージへの依存がある。
* Ubuntu の標準インストール状態では不足する場合があるため、検証開始前に導入確認を推奨する。

#### WV-03-02 Linux起動

確認結果

- 未実施

判定

- 未判定

#### WV-03-03 Dock共存

確認結果

- 未実施

判定

- 未判定

#### WV-03-04 WebView表示

確認結果

- 未実施

判定

- 未判定

#### WV-03-05 Child Window追従

確認結果

- 未実施

判定

- 未判定

#### WV-03-06 Floating禁止方式

確認結果

- 未実施

判定

- 未判定

### 評価

WV-03-01 Linuxビルドは合格。

Windows依存コードの platform 層移動後、
Linuxビルド環境での依存解決まで確認した。

ただし、WV-03-02以降は未実施であり、
WV-03全体の判定は保留とする。

### 後続検証

- WV-04 macOS egui共存

## WV評価

### 判定

未判定

### 根拠

WV-03-01 Linuxビルドは合格。

WV-03-02 Linux起動以降の検証が未実施のため、
WV-03全体としては未判定。

## 次工程

### WV-04 macOS egui共存

確認事項

- cargo build
- アプリ起動
- Dock矩形取得
- Dock移動
- Dockリサイズ
- WebView生成
- URL表示
- Child Window追従
- Floating禁止方式

## 備考

Linux では wry が WebKitGTK に依存する。

事前に以下の導入を確認すること。

Ubuntu系

```bash
sudo apt install \
  build-essential \
  pkg-config \
  libglib2.0-dev \
  libgio-2.0-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev
```

本検証では WebView の成立性だけでなく、Workflow IDE Framework の Support Panel 実装方式として Linux 上で継続採用可能かを評価対象とする。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-03 Linux egui共存
