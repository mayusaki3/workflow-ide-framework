<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261008-kairi-workspace-implementation-design
lang: ja-JP
canonical_title: Workspace 実装設計
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > 仕様 > Workspace > 実装設計

# Workspace 実装設計（実装前）

## 1. 現行コードとの相違

- `src/lib.rs` の `ApplicationConfig.layout: Option<LayoutConfig>` はApplication単位の単一Layoutであり、Workspace別Layoutを保持しない。
- `src/layout.rs` の `LayoutConfig` はPanel ID中心であり、Workspace ID、Container ID、Panel instance IDを区別しない。
- `build_dock_state()` は `DockState<String>` を生成し、空のroot panelを拒否する。新仕様では空のDefault Containerを有効とする必要がある。
- `PanelDefinition` は定義IDを持つが、複数instanceの識別は別途必要。
- `src/framework_settings.rs` の `FrameworkSettings` は現在Resource関連設定が中心であり、WorkspaceごとのUser Layoutを持たない。

## 2. 責務分離

| 層 | 責務 |
|---|---|
| Workspace registry | stable Workspace ID、Default Workspace、選択状態、lifetime |
| Container registry | Workspace所属、Default Container、通常/Floatingの区別 |
| Panel registry | Panel definitionとinstanceの識別・所有 |
| Layout model | Container配置、Panel配置、分割、選択、表示、Floating幾何情報 |
| Dock adapter | Layout modelとegui_dock等の表示状態の変換 |
| Persistence | Project内User設定のWorkspace別保存・復元 |

Dock backendのTree/Nodeは公開APIまたは保存形式に使用しない。

## 3. Rust公開API案

以下は型・methodの候補であり、仕様テストを満たすことを優先して実装時に確定する。

```rust
pub const DEFAULT_WORKSPACE_ID: &str = "framework.workspace.default";

pub struct WorkspaceId(String);
pub struct ContainerId(String);
pub struct PanelInstanceId {
    pub definition_id: String,
    pub instance_id: String,
}

pub enum ContainerKind {
    Normal,
    Floating,
}

pub enum PlacementTarget {
    Container(ContainerId),
    Floating { position: [f32; 2], size: [f32; 2] },
}

pub enum ShowTarget {
    Previous,
    Container(ContainerId),
}

pub struct WorkspaceRegistry { /* private */ }

impl WorkspaceRegistry {
    pub fn register(&mut self, id: WorkspaceId, name: String) -> Result<(), WorkspaceError>;
    pub fn unregister(&mut self, id: &WorkspaceId) -> Result<(), WorkspaceError>;
    pub fn select(&mut self, id: &WorkspaceId) -> Result<(), WorkspaceError>;
    pub fn add_container(&mut self, workspace: &WorkspaceId, id: ContainerId) -> Result<(), WorkspaceError>;
    pub fn place_panel(&mut self, workspace: &WorkspaceId, panel: &PanelInstanceId, target: PlacementTarget) -> Result<(), WorkspaceError>;
    pub fn move_panel(&mut self, workspace: &WorkspaceId, panel: &PanelInstanceId, target: PlacementTarget) -> Result<(), WorkspaceError>;
    pub fn hide_panel(&mut self, workspace: &WorkspaceId, panel: &PanelInstanceId) -> Result<(), WorkspaceError>;
    pub fn show_panel(&mut self, workspace: &WorkspaceId, panel: &PanelInstanceId, target: ShowTarget) -> Result<(), WorkspaceError>;
    pub fn reset_layout(&mut self, workspace: &WorkspaceId) -> Result<(), WorkspaceError>;
}
```

`ShowTarget::Previous` は同一Workspaceの直前配置を優先し、Floating履歴なら元の位置・サイズに復帰する。履歴が無効な場合はDefault Containerへ配置する。明示された無効Containerはエラーとする。

Floating Containerは1 Panel専用であり、他Panelのドラッグ＆ドロップを拒否する。Floating化では専用Containerを生成し、空になれば破棄する。Panel instanceは維持する。

## 4. 永続化案

Project内のKairi管理領域に、Workspace IDをキーとするUser Layoutを保存する。保存形式はbackend非依存とし、少なくともContainer配置、Panel instance参照、表示状態、Floating位置・サイズを保持する。

既存`framework_settings.toml`への格納と別ファイル化のいずれかは、既存Project保存処理との互換性を確認して決定する。既存形式を暗黙に破壊しない。

復元優先順位はUser Layout > Application既定Layout > Default Containerのみ。リセットは対象Workspaceのみ。

## 5. 実装順序

1. Workspace/Container/Panel instance識別型、Registry、失敗結果を実装する。
2. 実装前に確定した仕様テストWS-001以降を実装し、Registryの振る舞いを検証する。
3. Layout modelとDock adapterを分離し、空のDefault ContainerおよびContainer間移動を実現する。
4. Floating Containerの生成・破棄、ドロップ拒否、位置・サイズ復帰を実装する。
5. Project内User Layoutの保存・復元・対象Workspaceリセットを実装する。
6. Main WindowとProject dialogへ接続し、既存Applicationの回帰テストを行う。
7. 仕様テストの成功を確認した後、実装コードのカバレッジを計測し、補完テスト`WS-COV-xxx`を**別管理**で追加する。

## 6. 互換性と注意点

既存`ApplicationConfig.layout`はDefault WorkspaceのApplication既定Layoutへ対応付ける移行策を検討する。廃止・変更は利用アプリケーションへの影響を確認してから決定する。

現行のPanel IDとinstance IDの対応は既存Panel renderer/registryを調査して確定する。公開API案をそのまま実装可能と見なさない。

本書のコード例は設計案であり、コンパイル・テスト実行済みではない。

---

[目次](../目次.md) > 仕様 > Workspace > 実装設計
