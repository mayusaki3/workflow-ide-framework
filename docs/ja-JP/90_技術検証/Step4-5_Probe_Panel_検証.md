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
| library build | ？ |
| Probe Sample run | ？ |
| Probe PanelがDock内に表示 | ？ |
| Window / Host | ？ |
| Dock / Layout | ？ |
| WFIDE Logging | ？ |
| Lifecycle | ？ |
| Browser Surface | ？ |
| GPU Surface | ？ |
| Input / IME | ？ |
| Consumer PanelとProbe Panelの分離 | ？ |
| Linux VM | ？ |
| Linux実機 | ？ |
| macOS | ？ |

## Linux VM

Windows 11でStep 4.5成立後、同じcommitをLinux VMで検証する。

Linux VMでは最低限、build/run、Window、Dock、WFIDE Logging、Lifecycle、Probe Panel表示を確認する。Browser/GPU/Input/IMEはProbeへ統合されていない限り？のままとし、Phase 0の個別検証結果と混同しない。

Linux VMでGPU backendがllvmpipe/Vulkan software rendererの場合、その事実を記録し、物理Linux GPU検証とは扱わない。
