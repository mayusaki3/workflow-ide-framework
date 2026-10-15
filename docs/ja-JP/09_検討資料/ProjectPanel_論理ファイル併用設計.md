# Project Panel 論理リソース・ファイル併用設計（Meridian連携）

## 1. 目的と採用方針

Project Panelはアプリが定義する論理リソースと、既存Resource Registryが管理するファイルリソースの両方を表示する。MeridianのロボットモデルやSysIDモデルは例であり、Frameworkに固有種別名をハードコードしない。

## 2. 表示モデル

```text
Project
├─ Resources                         [Application]
│  ├─ Robot Models                    [Application-defined category]
│  │  └─ robot01                      [Logical resource]
│  │     ├─ robot.urdf                [File reference, optional]
│  │     └─ meshes/                  [Folder reference, optional]
│  └─ SysID Models                    [Application-defined category]
│     └─ sysid01                      [Logical resource]
└─ Project Settings                   [Framework]
```

上記ファイル名は例示であり、Meridianの保存形式を規定しない。論理リソースはファイル子要素を持たなくてもよい。ファイルを表示するか、どの階層まで展開するかはアプリが指定する。

## 3. 識別子と所有権

- 論理リソース: アプリが安定したID、表示名、種別、親子関係を提供する。表示名やファイルパスをIDの代わりにしない。
- ファイルリソース: 既存の `ResourceReference`（Application / Project / External）と `ResourceRegistry` のIDを再利用する。
- 論理リソースとファイルリソースは別のID名前空間とし、同名でも衝突させない。
- Frameworkはツリー表示、選択、展開状態、共通メニューの組み立てを担当する。Applicationは論理リソースの作成・編集・削除、関連ファイルの意味づけを担当する。
- Project SettingsはFrameworkが提供する独立したルートとし、アプリのリソース種別に含めない。
- Applicationの内部データの永続化は既存Application Project Adapterの責務とする。Frameworkは `application/` の内容を解釈しない。

## 4. ツリーと操作の契約

- カテゴリ、論理リソース、ファイル参照、フォルダー参照、Framework設定を区別できるノード種別を設ける。
- 論理リソースを展開すると、アプリが公開した子ノード（論理リソースまたはファイル参照）を表示する。
- ファイルを物理的に再帰走査して論理リソースに自動所属させない。未登録ファイルは既存Unregistered検索で扱う。
- ファイル参照は同じResource Registry entryを複数の論理ノードから表示してもよい。表示ノードIDとRegistry resource_idは分離する。
- 単一選択・複数選択、ダブルクリック、右クリックメニューはノード種別に応じてアプリへ通知する。アプリが扱わない共通ファイル操作は既存Resource Operation契約に従う。
- 変更操作は既存Journal、Dirty、Save/Openの契約を破らない。ファイル移動や置換時にはRegistry/参照の整合性を維持する。

## 5. 実装前のテストケース

| ID | 検証内容 | 期待結果 |
|---|---|---|
| PP-HYB-001 | アプリ定義カテゴリと論理リソース | 指定階層で表示 |
| PP-HYB-002 | 論理リソース配下にファイルとフォルダー | 子として表示・展開 |
| PP-HYB-003 | 子ファイルのない論理リソース | 正常表示 |
| PP-HYB-004 | 同じファイル参照を複数論理ノードに配置 | 表示ノードは別、Registry IDは同一 |
| PP-HYB-005 | 同名の論理リソースとファイル | ID衝突しない |
| PP-HYB-006 | Missing / RootUnavailable | 参照を保持し状態を区別 |
| PP-HYB-007 | アプリによる表示名変更 | 論理リソースIDは不変 |
| PP-HYB-008 | Project Save/Open | アプリデータ復元後にツリー再構築 |
| PP-HYB-009 | 未登録ファイル | 自動所属・自動登録しない |
| PP-HYB-010 | Framework Project Settings | アプリ定義カテゴリと独立して表示 |

## 6. 実装順序

1. 既存 `ProjectPanelModel` と `TreeModel` の利用範囲を確認し、上記テストを先に追加する。
2. アプリ定義カテゴリ・論理リソース・子ファイル参照を受ける表示モデルを追加する。既存 `ProvidedResource` / `ResourceProviderKind` の公開互換性を維持する。
3. Meridianから論理リソース一覧と子参照を渡せるAPIを追加する。
4. 選択・ダブルクリック・コンテキストメニューを接続する。
5. Save/Open・Registry・Journalとの統合検証を行う。

## 7. 未確定事項

- Meridianのロボットモデル／SysIDモデルの内部保存形式と子ファイルの実体。
- 論理リソースの削除時に関連ファイルを削除するか、独立して残すか。
- 大量リソース時の遅延展開APIの必要性。
- ツリー展開状態をProject設定に保存するか、User設定に保存するか。

これらはMeridian側とのすり合わせで決め、Framework固有の前提として先に固定しない。
