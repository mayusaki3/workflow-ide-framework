[日本語](./README.md) | [English](./README_en-US.md)

# Workflow IDE Framework

Workflow IDE Framework は、Rust で IDE 型アプリケーションを構築するためのアプリケーション組み込み型 Framework です。
Consumer Application が Framework の Panel、Layout、Project、Resource、Command 等の共通機能を利用し、domain 固有機能を追加する構成を想定しています。

> **Status: v0.1.0 development**
>
> v0.1.0 は公開準備中です。Windows / Linux / macOS を対象とし、未検証項目は公開検証表で `？` として明示します。

## 想定用途

- Runtime / Simulation IDE
- AI / Asset Studio
- Workflow Editor
- GPU Viewport を持つ開発・検証アプリケーション

## v0.1.0 の範囲

現在の実装と v0.1.0 仕様では、次の共通基盤を整備しています。

- Application / Panel 構成
- Dock / Layout 基盤
- Text Editor / Log Viewer / Tree Viewer
- Flow / Graph Editor
- Property Panel
- Controller Panel
- Logging
- Language / Theme / Font 設定
- Project persistence
- Resource Registry / File Resource Selector
- Application / Project / External Resource の区別
- Resource filesystem operation の整合性管理
- Notification
- Consumer Application 向け拡張境界

Project、Resource、Panel Container、Notification の一部は v0.1.0 向け設計・実装・検証中です。README上の項目だけを完成済み機能とはみなさず、検証状態はドキュメントの検証表を参照してください。

## 責務境界

Framework は IDE 共通機能と汎用 model/action/API を所有します。Consumer Application は domain model、domain validation、Application 固有 Project data、Resource の意味、Runtime 固有処理を所有します。

Framework を利用するために Consumer が Framework 内部型を fork / 直接変更することを前提としません。

## 対象OS

| OS | v0.1.0方針 |
| --- | --- |
| Windows | 実機検証 |
| Linux | VMを含め検証。実機必須項目は未検証として明示 |
| macOS | 環境を用意できない項目は未検証として明示 |

検証状態は `○ / ✕ / ？` で公開します。細かなUI調整は v0.1.0 の完了条件に含めず、機能成立、公開API、永続化、状態遷移、Consumer統合を優先します。

## 開発と公開物

`develop` は v0.1.0 開発ブランチです。開発中の `stepX_...` Example 名や技術検証資料は開発履歴として扱い、v0.1.0 の Consumer 向け導線では用途ベースの Example / ドキュメントへ整理します。

## Sample Application

Consumer向け機能は1つの統合Sample Applicationで確認できます。

```text
cargo run --example sample_application
```

- [Sample Application ガイド](./docs/ja-JP/03_利用ガイド/01_Sample_Application.md)

既存の `stepX_...` Exampleは開発・回帰検証履歴として当面保持し、公開Sampleの主導線には使用しません。

## ドキュメント

- [ドキュメント目次](./docs/ja-JP/目次.md)
- [要件定義](./docs/ja-JP/01_要件定義/要件定義目次.md)
- [仕様](./docs/ja-JP/02_仕様/仕様目次.md)
- [技術検証](./docs/ja-JP/90_技術検証/技術検証目次.md)
- [v0.1.0 公開準備チェックリスト](./docs/ja-JP/90_技術検証/v0.1.0_公開準備チェックリスト.md)

## v0.1.0 で固定しないもの

Panel の最小サイズ、Split の細かな既定比率、Notification の細かな表示挙動など、Application APIやProject互換性へ影響しないUI調整値は v0.1.0 で固定しません。実際に動作させた結果、必要なものを後続版で調整します。

## ライセンス

MIT License。詳細は [LICENSE](./LICENSE) を参照してください。

## Documentation specification

設計・仕様文書は [HLDocS](https://github.com/mayusaki3/HLDocS) を採用しています。
