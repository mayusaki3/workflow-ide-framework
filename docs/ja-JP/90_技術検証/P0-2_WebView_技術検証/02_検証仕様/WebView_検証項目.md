<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000002Z-WV21
lang: ja-JP
canonical_title: WebView 検証項目
document_type: test_spec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証項目

# WebView 検証項目

## 目的

P0-2 WebView 技術検証において、WebView Support Panel 構造の成立性を確認するための検証項目を定義する。

本検証では、WebView の単体表示だけでなく、Workflow IDE Framework の構成要素である egui、Command System、Event Bus との連携可能性を確認する。

## 検証方針

- 検証番号は `WV-XX` 形式で付与する。
- P0 段階では、実装品質ではなく成立性を確認する。
- 検証結果は `03_検証結果/WebView_検証結果.md` に記録する。
- WebView から IDE 操作を行う場合は、Command Bridge を経由する。

## 検証環境

| 項目 | 内容 |
|---|---|
| 対象 OS | Windows / Linux / macOS |
| 言語 | Rust |
| GUI | egui |
| Window Backend | eframe |
| WebView | wry |
| 通信 | Rust と WebView 間の双方向通信 |

## 検証項目一覧

| 検証番号 | 検証項目 | 概要 | 成功条件 |
|---|---|---|---|
| WV-01 | WebView 表示 | 単一 WebView を表示する | HTML が表示される |
| WV-02 | egui 共存 | egui UI と WebView を共存させる | 両方が同時に操作可能である |
| WV-03 | Multi Panel | 複数 WebView を表示する | 複数 WebView が個別に表示される |
| WV-04 | Focus 切替 | egui と WebView 間で Focus を切り替える | 入力対象が正しく切り替わる |
| WV-05 | Rust から WebView への通知 | Rust 側から WebView 側を更新する | WebView 側の表示または状態が更新される |
| WV-06 | WebView から Rust への通知 | WebView 側から Rust 側へ通知する | Rust 側でイベントを受信できる |
| WV-07 | Command Bridge | WebView から IDE Command を呼び出す | Command として処理される |

## WV-01 WebView 表示

### 目的

単一 WebView を生成し、HTML を表示できることを確認する。

### 手順

1. WebView を 1 つ生成する。
2. 検証用 HTML を読み込む。
3. 表示内容を確認する。

### 期待結果

- WebView が表示されること。
- 検証用 HTML の文字列または UI が確認できること。
- アプリケーションが異常終了しないこと。

### 検証ケース

| 検証番号 | 検証内容 | 成功条件 |
|---|---|---|
| WV-01-01 | WebView生成 | WebView生成成功 |
| WV-01-02 | URL表示 | example.com表示成功 |
| WV-01-03 | Hide/Show | 非表示後に正常復帰する |
| WV-01-04 | Dock追従 | Dock移動・Dockリサイズ後も表示位置が正しい |

## WV-02 egui 共存

### 目的

egui UI と WebView が同一アプリケーション内で共存できることを確認する。

### 手順

1. egui UI を表示する。
2. WebView を表示する。
3. egui 側のボタンや入力欄を操作する。
4. WebView 側のボタンや入力欄を操作する。

### 期待結果

- egui UI が操作可能であること。
- WebView UI が操作可能であること。
- 片方の操作により、もう片方が操作不能にならないこと。

## WV-03 Multi Panel

### 目的

複数 WebView を Support Panel として同時利用できることを確認する。

### 手順

1. WebView A を生成する。
2. WebView B を生成する。
3. それぞれ異なる HTML を読み込む。
4. WebView A と WebView B を個別に操作する。

### 期待結果

- WebView A と WebView B が同時に表示されること。
- それぞれ異なる表示内容を保持できること。
- 一方の操作が他方の表示や状態を破壊しないこと。

## WV-04 Focus 切替

### 目的

egui と WebView、または複数 WebView 間で Focus を切り替え可能であることを確認する。

### 手順

1. egui 入力欄を選択する。
2. WebView A の入力欄を選択する。
3. WebView B の入力欄を選択する。
4. 再度 egui 入力欄を選択する。
5. キーボード入力がどこに反映されるか確認する。

### 期待結果

- Focus 対象に入力が反映されること。
- Focus が外れた UI に入力が反映されないこと。
- Focus 切替時にアプリケーションが異常終了しないこと。

## WV-05 Rust から WebView への通知

### 目的

Rust 側から WebView 側へ通知し、WebView の表示または状態を更新できることを確認する。

### 手順

1. WebView に検証用 HTML を読み込む。
2. Rust 側から WebView 側へ通知する。
3. WebView 側で表示または状態を変更する。
4. 変更結果を確認する。

### 期待結果

- Rust 側から WebView 側へ通知できること。
- WebView 側の表示または状態が更新されること。

## WV-06 WebView から Rust への通知

### 目的

WebView 側から Rust 側へ通知できることを確認する。

### 手順

1. WebView に検証用 HTML を読み込む。
2. WebView 側の検証用 UI を操作する。
3. WebView 側から Rust 側へメッセージを送信する。
4. Rust 側で受信ログまたは状態変化を確認する。

### 期待結果

- WebView 側から Rust 側へメッセージが送信されること。
- Rust 側でメッセージを受信できること。
- 受信内容をログまたは状態として確認できること。

## WV-07 Command Bridge

### 目的

WebView から IDE Command を呼び出す構造が成立することを確認する。

### 手順

1. WebView に検証用 HTML を読み込む。
2. WebView 側から検証用 Command 名と引数を送信する。
3. Rust 側で Command として解釈する。
4. Command の実行結果を確認する。

### 期待結果

- WebView から送信した操作が Command として処理されること。
- Command 実行結果をログまたは UI 表示で確認できること。

## 成功判定

P0-2 WebView 技術検証は、WV-01 から WV-07 までの検証項目がすべて成功した場合に成功とする。

一部の OS で失敗した場合は、OS ごとの制約として `03_検証結果/WebView_検証結果.md` に記録する。

## 失敗時の扱い

以下の場合は、代替案の検討対象とする。

- WebView と egui の共存が困難な場合
- Focus 管理に重大な問題がある場合
- WebView と Rust の双方向通信が安定しない場合
- Command Bridge が成立しない場合
- OS ごとの差異が大きく、共通 Support Panel として扱いにくい場合

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > WebView 検証項目
