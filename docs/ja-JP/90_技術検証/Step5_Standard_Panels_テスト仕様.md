# Step 5 Standard Panels テスト仕様

## 1. 方針
テストを3層に分ける。

1. **自動テスト**: model、builder、adapter、domain非依存logic。 `cargo test`
2. **Framework Sample手動テスト**: Dock/描画/IME/drag/scroll/zoom等GUI interaction。
3. **Consumer Acceptance**: meridian-mujoco-runtimeから公開APIだけを使用して成立すること。

自動化できるlogicをSample目視だけに依存させず、GUI/OS依存項目をunit testのcoverage値だけで代替しない。

## 2. 自動テスト
`tests/step5_models.rs` をStep 5の最低regression suiteとする。

| ID | 対象 | 期待 |
|---|---|---|
| S5-A01 | TextDocument | 日本語text/metadata保持 |
| S5-A02 | TreeModel | 日本語label/階層保持 |
| S5-A03 | FlowModel | 日本語Node/Port、Edge、view初期値保持 |
| S5-A04 | Flow→Property | Node property生成とPropertyAction逆反映 |
| S5-A05 | Edge→Property | endpoint read-only property生成 |

## 3. GUI Sampleテスト
| ID | Step | 主確認 |
|---|---|---|
| S5-G01 | 5.5 | 日本語表示/IME、caret、scroll、wrap、status |
| S5-G02 | 5.6 | structured log、level色、auto/horizontal scroll |
| S5-G03 | 5.7 | 日本語Tree、展開、選択 |
| S5-G04 | 5.8 | 日本語Node、Node drag、Edge追従、接続/拒否/削除、選択解除 |
| S5-G05 | 5.8 | Canvas drag、Scrollbar、wheel zoom、文字scale、操作継続 |
| S5-G06 | 5.9 | 日本語Property、Text/Bool/Integer/Float/Enum、read-only |
| S5-G07 | 5.9 | Flow Node/Edge→Property、Property→Flow双方向反映 |

Sampleは日本語fontを設定し、日本語文字列を最低1箇所含める。

## 4. Consumer Acceptance
MeridianではFramework内部型へ依存せず、Application/Panel/Layoutと各Step 5公開model/actionからUIを構成する。simulation/HIL loopをegui frame lifetimeへ拘束しない。domain validationはMeridian側に置く。

## 5. OS検証
Windows実機、Linux VM、Linux実機、macOSを分ける。環境がない項目は `？` とし、非対応を意味する `✕` にしない。

## 6. Coverage
### 6.1 計測
Rust source coverageは `cargo llvm-cov` を標準候補とする。

```powershell
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --all-targets --summary-only
cargo llvm-cov --workspace --all-targets --html
```

### 6.2 判定
- line/function/region coverageを記録する。
- 現段階では数値thresholdを合否条件にしない。まずbaselineを取得する。
- GUI renderer、OS event、IME、native window、drag/scroll/zoomは自動coverage率が低くても手動test matrixで補完する。
- 新規domain非依存logicは原則として自動テスト対象にする。
- Coverage値をGUI acceptanceの代替にしない。

## 7. Meridian依頼前Gate
- `cargo test` 成功
- Step 5 sample build成功
- Windowsの既確認項目をtest matrixへ転記
- 日本語表示/入力確認
- coverage baseline取得
- 未検証項目を `？` として明示
- Consumerへ渡す公開API/非対象範囲を仕様と一致させる
