# Step 4.5 Framework Probe Panel 検証

## 目的

Framework + Sampleを各versionで継続検証するため、Framework自身の診断結果をDock内のProbe Panelへ集約する。

Probe PanelはFramework保守・検証専用であり、Meridian等のConsumerへ組み込む機能ではない。Consumer側検証とは分離する。

## 構成

```text
Probe Core
├─ Window / Host
├─ Dock / Layout
├─ WFIDE Logging
├─ Browser Surface
├─ GPU Surface
├─ Input / IME
└─ Lifecycle
       ↓
   Probe Panel
```

結果記号は `○ / ✕ / ？` のみ使用する。

現時点でStep 4.5 Hostから直接成立確認できる項目だけを○とする。Phase 0で個別に成立済みであっても、このProbe Panelからまだ接続・実行していないBrowser Surface / GPU Surface / Input / IMEは自動的に○へしない。

## 現在の自動判定

- Window / Host: Host UI到達時 ○
- Dock / Layout: ProbeをDock内で描画できた場合 ○
- WFIDE Logging: in-memory bufferにeventが存在すれば ○
- Lifecycle: Application -> Logging -> Host経路到達で ○
- Browser Surface: ？
- GPU Surface: ？
- Input / IME: ？

Surface/Input系は後続で既存Phase 0 probeとの統合を行う。

## Framework-only

`Application::framework_probe_panel()` はFramework Sample / verification build用入口とする。通常Consumerは呼び出さない。

Probe Panel IDはFramework内部予約ID `__wfide_probe` とし、Consumerのstable Panel IDとして使用しない。

## Windows 11 検証

```powershell
cargo build --lib
cargo run --example step4_5_probe_panel
```

| 項目 | 結果 |
| --- | --- |
| library build | ○ |
| Probe Sample run | ○ |
| Probe PanelがDock内に表示 | ○ |
| Window / Host | ○ |
| Dock / Layout | ○ |
| WFIDE Logging | ○ |
| Lifecycle | ○ |
| Browser Surface | ？ |
| GPU Surface | ？ |
| Input / IME | ？ |
| Consumer PanelとProbe Panelの分離 | ○ |
| Linux VM | ○ |
| Linux実機 | ？ |
| macOS | ？ |

## Linux VM

Windows 11でStep 4.5成立後、同じcommitをLinux VMで検証する。

Linux VMでは最低限、build/run、Window、Dock、WFIDE Logging、Lifecycle、Probe Panel表示を確認する。Browser/GPU/Input/IMEはProbeへ統合されていない限り？のままとし、Phase 0の個別検証結果と混同しない。

Linux VMでGPU backendがllvmpipe/Vulkan software rendererの場合、その事実を記録し、物理Linux GPU検証とは扱わない。

## Windows 11 実機確認結果

2026-09-19、`step4_5_probe_panel` をWindows 11で実行し、Dock内の `WFIDE Probe` Panel表示を確認した。

Probe表示結果:

- Window / Host: ○
- Dock / Layout: ○
- WFIDE Logging: ○
- Browser Surface: ？
- GPU Surface: ？
- Input / IME: ？
- Lifecycle: ○

Runtime Control / Runtime Status / Simulation View / LogはConsumer相当Sample Panelとして維持され、WFIDE ProbeはFramework maintenance diagnosticsとして別Panelに表示された。したがってConsumer PanelとProbe Panelの分離も○とする。

Browser Surface / GPU Surface / Input / IMEはStep 4.5 Hostからまだ実行していないため？を維持する。

Windows 11でのStep 4.5基本Probe Panel成立性確認は完了。次に同一Framework revisionをLinux VMで検証する。

## Linux VM 実機確認結果

2026-09-19、Ubuntu Linux VM上でStep 4.5 Probe Panel Sampleを実行し、GUI表示を確認した。

- Probe Sample run: ○
- Window / Host: ○
- Dock / Layout: ○
- WFIDE Logging: ○
- Lifecycle: ○
- Probe Panel表示: ○
- Consumer相当Sample PanelとProbe Panelの分離: ○
- Wayland経路でのWindow/event loop動作: ○（実行ログ・画面確認）
- Browser Surface: ？
- GPU Surface: ？
- Input / IME: ？

画面上で `WFIDE Probe` がDock内に表示され、Window / Host、Dock / Layout、WFIDE Logging、Lifecycleが○となることを確認した。Runtime Control / Runtime Status / Simulation View / Logも維持されている。

Browser Surface / GPU Surface / Input / IMEはStep 4.5 Hostから未実行のため？を維持する。Linux VMでのStep 4.5基本Probe Panel成立性は○とする。

なお、本確認はLinux VMであり、Linux物理実機の検証結果にはしない。GPU rendererについても本Step 4.5画面のみから物理GPU/llvmpipeの判定は行わない。Phase 0で確認済みのllvmpipe/Vulkan結果とは区別する。

## Logging Settings / 日本語表示の追加検証

Step 4.5 SampleにはFramework標準のLogging Settings Panelを含める。

- Runtime Log Levelの既定値はINFO。
- ERROR / WARN / INFO / DEBUG / TRACEを実行中に変更可能。
- level変更イベントはINFOとして記録する。現在levelがWARN/ERRORの場合は、排他下で一時的にINFOを有効化して変更イベントを記録してから要求levelへ切り替える。
- ERROR: エラーのみ。
- WARN: 警告とエラー。
- INFO: 通常の動作情報、警告、エラー。既定値。
- DEBUG: INFOに加えてデバッグ情報。
- TRACE: DEBUGに加えて最も詳細な内部処理。ログ量・処理負荷増大に注意する。

P0-1b EmbeddedFont技術検証の成果をFramework Applicationへ接続し、`AppearanceConfig::font_path` でeguiへアプリケーションfontを登録できるようにする。Step 4.5 SampleではP0-1bのNotoSansCJK-Regular.ttcを指定し、Logging Settings内の日本語説明を実際のFramework Sampleで確認する。

font asset本体は従来方針どおりrepositoryへ含めないため、未配置環境ではP0-1bのsetup_fontsスクリプトで準備する。読み込み失敗時はWARNを記録し、egui既定fontで継続する。

この追加部分のWindows/Linux VM表示結果は再確認後に○/✕/？を更新する。

### Windows 日本語表示 再確認

2026-09-20、Framework default font assetをsetup後、Windows 11のStep 4.5 SampleでLogging Settingsを表示し、日本語説明が欠字（□）なしで描画されることを画面確認した。

- Framework default font経由の日本語表示: ○
- Logging Settings日本語説明: ○
- ERROR / WARN / INFO / DEBUG / TRACE説明表示: ○
- Dock内での日本語描画: ○

前回の欠字表示はfont asset未配置によるもので、Framework側の `assets/fonts/default/NotoSansCJK-Regular.ttc` を準備した状態では解消した。Linux VMでの同経路は未再確認のため？を維持する。

### Linux VM 日本語表示 再確認

2026-09-20、Ubuntu Linux VMでFramework default font assetをsetup後、Step 4.5 SampleのLogging Settingsを表示し、日本語説明が欠字なしで描画されることを画面確認した。

- Font asset存在確認（Font OK）: ○
- Framework default font load: ○（`wfide::font: application font loaded path=assets/fonts/default/NotoSansCJK-Regular.ttc`）
- Logging Settings日本語説明: ○
- ERROR / WARN / INFO / DEBUG / TRACE説明表示: ○
- Dock内での日本語描画: ○
- Runtime Log Level既定値 INFO: ○

本結果はUbuntu Linux VM上の確認であり、Linux物理実機の結果にはしない。

### Linux VM 操作感

Runtime Log Levelの既定値をTRACEからINFOへ変更した状態でも、Ubuntu Linux VM上の主観的な操作感には明確な改善を確認できなかった。

- INFO既定化: ○（設定・ログ出力量の抑制は確認済み）
- GUI操作感の改善: ？（明確な差を確認できず）
- 原因: 未確定。VM環境による描画・仮想GPU等の影響が候補だが、本結果だけでは断定しない。

したがって、以前の重さをTRACE/DEBUGログ出力だけに起因するとは判断しない。Linux物理実機または追加の性能計測が可能になるまで、性能面は未検証として扱う。

## Step 4.5 判定

Step 4.5 の最小マイルストーンは **完了** とする。

完了根拠:

- Framework-only Probe Panel: Windows 11 ○ / Linux VM ○
- Window / Host、Dock / Layout、WFIDE Logging、Lifecycle: Windows 11 ○ / Linux VM ○
- Consumer相当Sample PanelとProbe Panelの分離: Windows 11 ○ / Linux VM ○
- Framework標準 Logging Settings Panel: Windows 11 ○ / Linux VM ○
- Runtime Log Level既定INFOおよび実行中切替: Windows 11 ○
- Embedded FontによるFramework UI日本語表示: Windows 11 ○ / Linux VM ○
- Linux VM性能: ？（機能成立性とは分離）
- Linux物理実機 / macOS: ？

Browser Surface / GPU Surface / Input / IME はStep 4.5 Hostへ未統合のため？を維持する。これらはPhase 0での個別成立結果をStep 4.5の○へ読み替えない。Probe Coreへの追加統合は継続課題とし、Step 5のPanel実装API設計を開始することを妨げない。

Step 5以降もFramework + SampleとConsumerの二系統検証を継続する。
