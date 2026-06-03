<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-04 Linux WebView方式選定
document_type: note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-04 Linux WebView方式選定

# WV-04 Linux WebView方式選定

## 目的

WV-03 にて Linux Wayland 環境で
wry build_as_child() が成立しないことを確認した。

本検証では Linux 環境における
Workflow IDE Framework の
WebView Support Panel 実装方式を選定する。

本検証の結果をもって、

- Linux実装方式
- macOS検証方針
- Framework標準方式

を決定する。

## 前提条件

### WV-03

確認済み事項

- Linuxビルド成功
- Linux起動成功
- Dock共存成功
- build_as_child() は Wayland 非対応
- Waylandで UnsupportedWindowHandle 発生
- eframe から GTK Container 取得方法未確認

## 検証候補

### 候補A

eframe + wry(build_gtk)

概要

- eframe継続
- GTK FixedへWebView配置
- Wayland対応

期待効果

- Windows版との共通化維持
- Framework構造維持

懸念

- eframeからGTK Widget取得可否

### 候補B

tao + wry(build_gtk)

概要

- eframe廃止
- taoをRoot Window化
- GTK FixedへWebView配置

期待効果

- wry推奨構成
- Wayland正式対応

懸念

- egui統合方式変更

### 候補C

別Window方式

概要

- Support Panelを独立Window化

期待効果

- OS依存最小化

懸念

- IDE UX低下

### 候補D

別Process方式

概要

- WebView専用プロセス化

期待効果

- 高い分離性

懸念

- IPC追加
- 実装コスト増加

## PoC

### WV-04-01 eframe GTK取得可否

確認項目

- GTK Widget取得
- GTK Container取得
- build_gtk接続可否

合格条件

- eframe から GTK Container へアクセスできること
- build_gtk() に必要な Widget を取得できること
- Workflow IDE Framework の既存構造を維持できる見込みがあること

不合格条件

- GTK Container を取得できない
- 非公開API依存となる
- OS依存コードを Framework 上位層へ露出させる必要がある

### WV-04-02 build_gtk生成

確認項目

- WebView生成
- URL表示
- Wayland動作

合格条件

- build_gtk() により WebView を生成できること
- URL表示ができること
- Wayland環境で動作すること
- GTK Container 配下への配置が確認できること
- Child Surface として制御可能であること

### WV-04-03 Child Surface追従

確認項目

- Dock移動
- Dockリサイズ
- タブ切替
- Hide制御

合格条件

- Dock移動へ追従できること
- Dockリサイズへ追従できること
- タブ切替へ追従できること
- 非アクティブ時に退避できること
- Windows版と同等の抽象化が可能であること

### WV-04-04 方式評価

確認項目

- Windows共通化
- Linux対応性
- macOS展開性
- Framework実装容易性

## 実施結果

### WV-04-01 eframe GTK取得可否

確認結果

- 未実施

判定

- 未判定

### WV-04-02 build_gtk生成

確認結果

- 未実施

判定

- 未判定

### WV-04-03 Child Surface追従

確認結果

- 未実施

判定

- 未判定

### WV-04-04 方式評価

確認結果

- 未実施

判定

- 未判定

## 評価

未実施

## WV評価

### 判定

未判定

### 根拠

未実施

## 次工程

### WV-05 macOS WebView共存

## 備考

Linux版は
「Framework利用者がOS差異を意識しない」
ことを合格条件とする。

内部実装は、

- Child Window
- GTK Container
- Native Surface

のいずれでもよい。

重要なのは Framework 外部仕様の統一である。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](./検証目次.md) > WV-04 Linux WebView方式選定
