<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261008-kairi-workspace-tests
lang: ja-JP
canonical_title: Workspace テスト仕様
document_type: testspec
canonical_document: true
-->

[目次](../目次.md) > テスト仕様 > Workspace

# Workspace テスト仕様

## 1. 対象と目的

機能仕様第3章「Workspace」の仕様適合性を、実装構造に依存しない振る舞いとして検証する。

本書のテストは実装前に定義する仕様テストとし、実装後のコードカバレッジ補完テストとは分離する。

## 2. テスト区分と管理

- **仕様テスト（`WS-xxx`）**：要件・仕様から導出し、実装前に確定する。仕様変更がない限り、実装都合で期待結果を変更しない。
- **カバレッジ補完テスト（`WS-COV-xxx`）**：実装後、未網羅のコード経路を確認して追加する。実装変更に伴い追加・変更・削除できる。本書の仕様テストとは別管理とする。
- 仕様テストの成功と、単体テストのカバレッジ100%は独立した完了条件とする。
- 実装後に発見した仕様上の不備は、仕様変更の合意を経て仕様テストへ反映する。補完テストによって仕様の期待結果を上書きしない。

## 3. Workspace生成・識別

| ID | 条件・操作 | 期待結果 |
|---|---|---|
| WS-001 | Application初期化 | `framework.workspace.default` が存在する |
| WS-002 | Applicationが追加Workspaceを登録しない | Default Workspaceのみで利用できる |
| WS-003 | Applicationが独自Workspaceを追加 | Default Workspaceが維持され、追加Workspaceも利用できる |
| WS-004 | Default Workspaceの削除または登録解除を要求 | 拒否され、Default Workspaceが残る |
| WS-005 | Workspaceの表示名を変更 | Workspace IDと保存済みLayoutの対応は変化しない |
| WS-006 | 複数Workspaceを登録し選択 | 指定したWorkspaceが選択される |

## 4. ContainerとLayout設定

| ID | 条件・操作 | 期待結果 |
|---|---|---|
| WS-007 | Default Workspace生成 | Default Containerが1つ自動生成される |
| WS-008 | 追加Workspace生成 | 当該WorkspaceにDefault Containerが1つ自動生成される |
| WS-009 | Panelの配置先を指定せず配置 | Default Containerが配置先となる |
| WS-010 | Panelの配置先Containerを明示 | 指定Containerが優先される |
| WS-011 | 非表示Panelの再表示先を明示 | 指定Containerが優先される |
| WS-012 | Containerを追加し水平方向へ分割 | 指定した分割関係が反映される |
| WS-013 | Containerを追加し垂直方向へ分割 | 指定した分割関係が反映される |
| WS-014 | 分割比率を設定 | 指定した比率がLayoutに反映される |
| WS-015 | 同一Containerに複数Panelを配置して選択 | 選択中Panelが反映される |
| WS-016 | Panel表示状態を変更 | 対象WorkspaceのLayout状態に反映される |
| WS-017 | 未登録PanelをLayoutから参照 | 不正として扱われ、Panelを暗黙に生成しない |
| WS-018 | 存在しないWorkspaceまたはContainerを対象にLayout設定 | 対象を暗黙に生成しない |
| WS-019 | Applicationが既定Layoutを定義しない | Default Containerのみを持つ既定状態を使用できる |

## 5. Panel配置・所有・切替

| ID | 条件・操作 | 期待結果 |
|---|---|---|
| WS-020 | 登録済みPanelをどのWorkspaceにも配置しない | 登録状態を維持し、未配置を許容する |
| WS-021 | 同一Panel instanceを同一Workspaceに二重配置 | 重複配置を許可しない |
| WS-022 | 同一Panel instanceを異なるWorkspaceへ配置 | 両Workspaceで同じinstanceを参照できる |
| WS-023 | 同一Panel instanceを複数Workspaceで別Containerへ配置 | 配置状態がWorkspaceごとに独立する |
| WS-024 | Workspaceを切り替える | 切替先のLayoutとPanel表示状態が反映される |
| WS-025 | 同一Panel instanceを持つWorkspace間で切替 | Panel instanceが再生成・破棄されない |
| WS-026 | Panelの表示・非表示またはWorkspace切替 | PanelとApplication固有Runtimeのownership/lifetimeを暗黙に変更しない |
| WS-027 | Panelを配置していないWorkspaceへ切替 | 他WorkspaceのPanel登録状態・配置状態を破壊しない |

## 6. 永続化・復元・リセット

| ID | 条件・操作 | 期待結果 |
|---|---|---|
| WS-028 | Workspace Layoutを保存 | Project内のUser設定として保存される |
| WS-029 | 保存済みLayoutがあるWorkspaceを復元 | Application既定Layoutより保存済みLayoutが優先される |
| WS-030 | 保存済みLayoutがないWorkspaceを復元 | Application既定Layoutを使用する |
| WS-031 | 保存済みLayoutもApplication既定Layoutもない | Default Containerのみの既定状態を使用する |
| WS-032 | 表示中Workspaceを指定してリセット | 対象のみ既定Layoutに戻り、他Workspaceは変化しない |
| WS-033 | 非表示Workspaceを明示してリセット | 指定したWorkspaceのみリセットされる |
| WS-034 | Application既定LayoutがないWorkspaceをリセット | Default Containerのみの既定状態へ戻る |
| WS-035 | Application更新で既定Layoutを変更し、既存User Layoutを復元 | User Layoutを優先し、新既定Layoutを暗黙にマージしない |
| WS-036 | Application更新後に既定値へリセット | 更新後のApplication既定Layoutが適用される |

## 7. DialogでのWorkspace利用

| ID | 条件・操作 | 期待結果 |
|---|---|---|
| WS-037 | Dialog表示用Workspaceを生成 | 通常Workspaceと同じ機構でDefault Containerが生成される |
| WS-038 | Dialog内WorkspaceにContainerとPanelを配置 | 通常Workspaceと同じLayout操作が利用できる |
| WS-039 | DialogとMain Windowで別Workspaceを使用 | ID、Layout、配置状態が独立する |
| WS-040 | Dialogを閉じる | 表示終了だけではWorkspaceが暗黙に破棄されない |
| WS-041 | 所有者がWorkspaceを破棄 | Dialogの表示終了とは独立して所有者がlifetimeを管理できる |

## 8. 実装前の確認事項

以下は本テストの期待結果を変更するための項目ではなく、公開I/F設計時に具体化する事項である。

- Workspace/Containerの識別子、登録・生成・破棄APIおよび失敗結果の表現
- Panelの再表示先指定と、保存済み配置先との優先順位
- Project内User設定の保存形式・保存タイミング
- Dialog表示対象Workspaceの生成・所有・破棄の公開境界
- Dock backendを利用した場合の観測可能なLayout検証方法

## 9. 完了条件

- 仕様テスト`WS-001`～`WS-041`がすべて成功すること。
- Workspace関連の既存機能にregressionがないこと。
- 単体テストのカバレッジ100%を、別管理のカバレッジ補完テストを含めて確認すること。
- 利用アプリケーションがKairi内部実装へ依存せずWorkspaceを構成・利用できること。

---

[目次](../目次.md) > テスト仕様 > Workspace
