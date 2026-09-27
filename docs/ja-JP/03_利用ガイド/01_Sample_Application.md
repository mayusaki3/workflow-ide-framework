# Sample Application

## 1. 目的

v0.1.0のConsumer向けSampleは、機能ごとの独立Sampleではなく、一つのSample Applicationへ統合する。

Sample Application自体をFramework Consumerのreference implementationとして使用し、複数PanelやFramework機能を同時に使用した場合の統合動作も検証する。

## 2. 起動

```text
cargo run --example sample_application
```

## 2.1 Project lifecycle

通常起動時はProjectを自動生成しない。起動直後はProject未オープン状態とし、New Project / Open Projectから開始する。

New ProjectではSample ApplicationがSample用Flow、Controller、Text等の初期Application dataを生成する。Deferred Projectの場合、初回SaveでProject Rootを選択する。

これによりSample Application自身で、未オープン → New/Open → Save/Save As → Close/OpenというConsumerの標準Project lifecycleを検証する。

開発中に各Panelを即時確認する用途だけは `--demo-project` を使用し、固定Sample dataを読み込む。これはAcceptance用の通常起動とは分離する。

```text
cargo run --example sample_application -- --demo-project
```

現時点ではSample Application UIへのProject lifecycle接続は実装途中であり、通常起動の切替はProject UI/API接続時に行う。

## 3. 含める機能

- Project / Tree表示
- Flow / Graph Editor
- Property PanelとFlow連携
- Controller Panel
- Property PanelとController連携
- Text Editor
- Log Viewer
- Language Panel
- Theme Panel
- Dock / Layout
- 日本語表示

Project persistence、Resource Registry、Notification等は実装された時点で同じSample Applicationへ追加する。

## 4. 開発用Sample / Probe

既存の `stepX_...` Exampleはv0.1.0開発履歴・回帰検証用として当面保持する。Consumer向け公開導線では使用しない。

IME、window、logging等のProbeはFramework内部の技術検証用であり、Sample Applicationの機能として統合しない。

最終的に開発用Example/Probeを移動する場合は、既存の検証文書・CI・scriptからの参照を確認してから行う。

## 5. 検証方針

Sample Applicationは個々のPanel表示だけでなく、次を確認する。

- 複数Panelが同時に存在しても操作が干渉しない
- Flow / Controllerの選択がProperty Panelへ正しく反映される
- Text Editorで日本語表示・入力が成立する
- Language / Theme変更が同一Application内で成立する
- Log ViewerがFramework/Applicationのlogを表示できる
- Layout内の各Panelへアクセスできる

細かなPanelサイズやSplit比率はv0.1.0では調整対象としない。
