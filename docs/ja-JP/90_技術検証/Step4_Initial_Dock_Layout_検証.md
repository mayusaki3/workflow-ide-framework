# Step 4 Initial Dock Layout 検証

## 目的

ConsumerがPanel IDだけを使用して初期Dock Layoutを宣言でき、`egui_dock` のTree / NodeIndex等のbackend APIを直接扱わずにDock UIを構成できることを確認する。

## Framework API

`LayoutConfig` を公開し、以下をConsumer向け操作とする。

- root areaへ複数Panelを配置
- left / right / above / below split
- split ratio
- 同一areaの複数Panel（tab）
- initial selected Panel
- stable Panel IDによる参照

Layoutから未知Panel ID、initially hidden Panel、重複Panel IDを参照した場合はApplication起動前に検出する。

`egui_dock::DockState`、Tree、NodeIndex等はFramework内部実装とし、Consumer APIへ露出させない。

## backend選定

Frameworkが現在使用するegui 0.34系との互換性から、Step 4では `egui_dock 0.19.1` を使用する。Consumerはこのversionや型へ依存しない。

## Sample

`examples/step4_initial_dock_layout.rs`

想定配置:

```text
+------------------+--------------------------------+
| Runtime Control  | Simulation View / Help(tab)    |
+------------------+                                |
| Runtime Status   |                                |
+------------------+--------------------------------+
|                  | Log                            |
+------------------+--------------------------------+
```

実際のsplit境界はratioに従う。Sampleは、nested split、同一areaの複数Panel、selected Panelを同時に確認する。

Step 4からFramework内の仮表示名を `workflow-ide-framework v0.1.0 Sample` に整理する。

## Windows 11 検証

実行:

```powershell
cargo build --lib
cargo run --example step4_initial_dock_layout
```

| 項目 | 結果 |
| --- | --- |
| library build | ○ |
| Step 4 Sample run | ○ |
| Window起動 | ○ |
| Dock UI表示 | ○ |
| horizontal split | ○ |
| vertical split | ○ |
| nested split | ○ |
| 同一areaの複数Panel/tab | ○ |
| selected Panel | ○ |
| split resize操作 | ○ |
| Panel tab切替 | ○ |
| Consumer APIがPanel IDのみでlayoutを参照 | ○ |
| backend Tree / NodeIndex非公開 | ○（API構造確認） |
| Linux | ？ |
| macOS | ？ |

実機確認していない項目を○にしない。

## Step 4の範囲外

- Panel内部の本実装
- GPU Surface接続
- Browser Surface接続
- Layout永続化
- Workspace保存Layoutの復元
- Runtime process分離
- Log Panel Viewer接続

WFIDE LoggingのConsumer実環境確認は、Step 4 Framework検証完了後のMeridian Consumer検証指示へ併記する。

## Windows 11 実機確認記録

2026-09-19、Step 4 SampleをWindows 11で実行して確認した。

- 初期状態で Runtime Control / Runtime Status / Simulation View / Log のnested Dock構成を表示: ○
- Simulation View と Help の同一area tab構成: ○
- Simulation View / Help のtab切替: ○
- split境界のドラッグによるhorizontal / vertical resize: ○
- resize後も各Panel ID / kindを維持: ○
- Window縮小時にもDock構成を維持: ○
- Framework表示名が `workflow-ide-framework v0.1.0 Sample` に整理されていること: ○

Panelのclose操作、Dock移動、Layout永続化はStep 4の必須検証対象には含めない。
