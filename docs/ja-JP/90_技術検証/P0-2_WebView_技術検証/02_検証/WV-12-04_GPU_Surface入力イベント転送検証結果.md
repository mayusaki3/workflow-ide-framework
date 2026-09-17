<!--
HLDocS:LLM-MANAGED
lang: ja-JP
canonical_title: WV-12-04 GPU Surface入力イベント転送検証結果
document_type: testresult
canonical_document: false
-->

# WV-12-04 GPU Surface 入力イベント転送検証結果

## 目的

Dock 上の GPU Surface 領域で受けた入力イベントを GPU Surface 側へ転送できることを確認する。

## Windows 実機結果

`wgpu_surface_input_probe` を Windows 実機で実行した。

| 項目 | 結果 | 根拠 |
| --- | --- | --- |
| PointerMove | ○ | `pointer_move=2634` |
| PointerButton | ○ | `pointer_button=12` |
| Wheel | ○ | `wheel=37` |
| Keyboard / Text | ○ | `keyboard=20`。Key press/release および Text イベントを確認 |
| Focus | ○ | `focus=7`。Focus true / false の遷移を確認 |

終了時ログ:

```text
WV-12-04 result: pointer_move=2634, pointer_button=12, wheel=37, keyboard=20, focus=7
WV-12-04 GPU Surface input probe shutdown
```

キーボードについては `A`、`Enter`、矢印キーの press / release、および `Text "a"` を確認した。

Focus については GPU Surface のクリックによる `Focus true`、他 UI への遷移による `Focus false`、再フォーカスを確認した。

## 判定

Windows 実機について WV-12-04 の5項目をすべて `○` とする。

これにより Windows 実機では GPU Surface の Dock 表示、Resize / Visibility / Lifecycle、基本入力転送まで成立した。

Linux VM、Linux 実機、macOS は本結果では判定せず `？` のままとする。
