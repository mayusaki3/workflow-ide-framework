# Step 5 Standard Panels 仕様

## 1. 目的
Step 5は、ConsumerがIDEを構成するための汎用Standard UI Panel群を提供する。FrameworkはUI表示・入力・汎用model/action境界を所有し、MuJoCo、URDF、HIL等のdomain規則はConsumerが所有する。

## 2. 対象
| Step | 機能 | 公開境界 |
|---|---|---|
| 5.5 | Text Editor | TextDocument / TextEditorOptions / TextEditorAction |
| 5.6 | Log Viewer | LogViewerOptions + logging snapshot |
| 5.7 | Tree Viewer | TreeNode / TreeModel / TreeAction |
| 5.8 | Flow / Graph Editor | FlowNode / FlowPort / FlowEdge / FlowModel / FlowAction / ConnectionValidator |
| 5.9 | Property Panel | PropertyModel / PropertyGroup / PropertyItem / PropertyValue / PropertyAction |

## 3. 共通要求
- PanelはStandardUiとしてDock内へ配置できる。
- 日本語を含む表示・入力を扱える。Application fontはConsumerから指定可能で、Framework sampleでは `assets/fonts/default/NotoSansCJK-Regular.ttc` を使用する。
- Sampleには日本語文字列を必ず含め、日本語font regressionを目視検出できること。
- Framework model/actionはdomain objectを直接所有しない。
- Windows / Linux / macOSの検証状態は `○ / ✕ / ？` のみで記録する。

## 4. Text Editor
複数行Text、read-only、modified、language hint、encoding metadata、cursor line/column、word wrap、SaveRequestedを扱う。UTF-8/Shift_JISはmetadataであり、v0.1.0ではfile transcodingを保証しない。日本語IMEはegui/winit入力境界で扱う。

## 5. Log Viewer
Framework loggingのin-memory snapshotを表示する。level/target/time/message/fieldsの構造化情報を保持し、level別表示、auto-scroll、horizontal scrollを提供する。Viewerはlogging初期化・rotation・retentionを所有しない。

## 6. Tree Viewer
任意階層のTreeNodeを表示し、stable IDによる選択をTreeActionとして通知する。filesystem/project/domain treeの意味はConsumerが決める。

## 7. Flow / Graph Editor
Node/Port/Edgeを汎用graphとして扱う。Node移動、Node/Edge選択、空白選択解除、接続作成/拒否、Edge削除をActionとして通知する。Input/Outputのクリック順はFrameworkでOutput→Inputへ正規化する。Consumer validatorはdomain規則を判定する。Canvas dragとScrollbarは同一ScrollArea offsetを使い、wheelはzoomを行う。

## 8. Property Panel
Text/Bool/Integer/Float/Enumとread-onlyを表示・編集し、ValueChangedを通知する。Flow adapterはreference implementationであり、Consumer固有property schemaをFrameworkへ固定しない。

## 9. 非対象
Workspace完全永続化、Undo/Redo完成、domain validation、MuJoCo/URDF固有編集、Plugin API、Accessibility/DnD/Clipboard完全対応はStep 5の完了条件に含めない。
