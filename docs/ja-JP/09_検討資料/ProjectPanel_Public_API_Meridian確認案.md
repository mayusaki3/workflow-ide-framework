# Project Panel Public API 案 — Meridian確認用

状態: **提案・未確定**。Meridian側との合意前に実装・公開API確定を行わない。

## 1. 目的

アプリが定義する論理リソース（Robot / SysID等）と、既存の `ResourceRegistry` に登録されたファイルリソースを一つのProject Panelツリーに表示する。FrameworkがMeridianのファイル形式や内部モデルを解釈しない。

既存実装: `src/project_panel.rs` の `ProjectPanelModel`、`ProjectPanelNode`、`ProvidedResource` と `src/tree_viewer.rs` の `TreeModel`、`TreeAction` を拡張対象とする。既存の公開型を破壊的に変更しない。

## 2. Rust API（概念案）

以下は**コンパイル済みの実装ではない**。型名・イベントの粒度はMeridianの回答を受けて確定する。

```rust
// Application -> Framework: panel snapshot
pub trait ApplicationProjectPanelProvider {
    fn project_panel_roots(&self, project: &ProjectContext) -> Vec<AppPanelNode>;
    fn handle_project_panel_action(
        &mut self,
        project: &ProjectContext,
        action: AppPanelAction,
    ) -> AppPanelActionResult;
}

pub struct AppPanelNode {
    pub id: String,                 // stable application-owned node ID
    pub label: String,
    pub kind: AppPanelNodeKind,
    pub children: Vec<AppPanelNode>,
}

pub enum AppPanelNodeKind {
    Category { type_id: String },
    LogicalResource { resource_id: String, type_id: String },
    RegisteredResource { registry_resource_id: String },
    // Unregistered physical files are not implicitly added to Registry.
}

pub enum AppPanelAction {
    Selected { node_ids: Vec<String> },
    Activated { node_id: String }, // e.g. double-click
    ContextAction { node_ids: Vec<String>, action_id: String },
}

pub enum AppPanelActionResult {
    Handled,
    NotHandled,     // Framework may run an existing supported default action
    Rejected { reason: String },
}
```

`AppPanelNode.id` はツリー上の出現位置を識別し、`resource_id` / `registry_resource_id` とは別物とする。同一ファイルを複数の論理リソース配下に表示できる。登録済みファイルのscope/path/stateはRegistry側を正とし、表示用ノードに重複保持しない。

Frameworkが管理するProject SettingsルートはApplicationから受け取らず、Frameworkが追加する。

## 3. Meridianからの利用例

```rust
// Illustrative pseudocode; type names may change after review.
AppPanelNode {
    id: "meridian.category.robots".into(),
    label: "ロボットモデル".into(),
    kind: AppPanelNodeKind::Category {
        type_id: "meridian.robot.category".into(),
    },
    children: vec![AppPanelNode {
        id: "meridian.robot.robot01".into(),
        label: "robot01".into(),
        kind: AppPanelNodeKind::LogicalResource {
            resource_id: "robot01".into(),
            type_id: "meridian.robot".into(),
        },
        children: vec![AppPanelNode {
            id: "meridian.robot.robot01.file.urdf".into(),
            label: "robot.urdf".into(),
            kind: AppPanelNodeKind::RegisteredResource {
                registry_resource_id: "registered-urdf-resource-id".into(),
            },
            children: vec![],
        }],
    }],
}
```

SysIDモデルは `type_id: "meridian.sysid"` 等のアプリ定義種別として同様に登録する。URDFやファイルIDは例であり、実際のMeridian内部形式を要求しない。

## 4. 更新通知・ライフサイクル案

初期実装は**全量スナップショット再取得**を基準とし、差分通知・遅延展開は必要性を確認してから導入する。

- Project Open後、Applicationデータの読み込み・整合性確認が済んでからProviderを呼び出す。
- Applicationの論理リソース追加・削除・名称変更・参照変更時は、Frameworkへ `ProjectPanelInvalidated` 相当の通知を送り、スナップショットを再取得する（具体的な通知IFは既存イベント機構と調整）。
- スナップショット更新時はstable node IDに基づき選択・展開状態を可能な範囲で維持する。
- Saveは既存Application Project Adapterに委譲し、ツリー表示モデルを別途永続化しない。
- ファイル移動・削除・Locate/Replaceは既存Resource Operation/Journal/Registryに委譲する。論理リソースの削除はアプリ側の責務とし、関連ファイルの削除を暗黙に行わない。

## 5. 既存UIとの差分

現在の `TreeAction` は `Selected` のみ。Activated / ContextAction / 複数選択を追加するには `tree_viewer` のイベント拡張が必要。既存 `ProjectPanelNode::Logical` / `Resources`、`ProvidedResource` は既存利用者向けに残し、新APIを追加する方式を優先する。

## 6. Meridian側への確認事項

1. **論理リソースID**: Robot / SysIDにProject内で永続的なIDがあるか。表示名変更後もIDが不変か。
2. **子ファイル**: 論理リソース配下に表示するファイルは、既存Registryへ登録できるか。ファイル単体とフォルダー単位のどちらが必要か。
3. **ツリー階層**: Robot / SysID以外のカテゴリや、論理リソースの入れ子が必要か。
4. **操作**: 選択・ダブルクリック・右クリックのどのイベントが必要か。複数選択操作が必要か。
5. **更新タイミング**: 編集直後に通知できるか。全量再取得で性能上問題がないか。
6. **削除**: 論理リソース削除と関連ファイル削除を独立させる方針でよいか。
7. **保存**: Application Project AdapterのSave/Open後に一覧を再構築できるか。
8. **未登録ファイル**: ツリーに未登録ファイルを直接表示する必要があるか（現案では不可。既存Unregistered検索を使用）。
9. **Project Settings**: FWが独立ルートとして表示する構成で問題ないか。

## 7. 合意後の実装と検証

合意後に `PP-HYB-001`〜`PP-HYB-010` のテストを実装し、ツリー表示モデル、操作通知、Registry連携、Save/Open統合の順に進める。公開APIはMeridianの回答を反映してから確定する。
