# Step 3.5 WFIDE Logging 検証

## 目的

Step 4 Dock Layout より前に、Panel に依存しない WFIDE 共通 Logging 基盤を成立させる。

## 実装対象

- `tracing` を共通イベント基盤として使用する。
- Framework 自身と Consumer Application の双方から利用可能とする。
- console output。
- in-memory buffer（既定 2,000 行）。
- rotating file output。
- 日次 rotation。
- 保持ファイル数を設定可能とし、既定 7 世代とする。
- 既定出力先は `logs/`。
- `workflow-ide-framework` は利用側で `wfide` と alias 可能とし、`wfide::tracing` を公開する。
- Log Panel は Logging の所有者ではなく、後続 Step で in-memory buffer を購読・表示する Viewer とする。

## Sample

`examples/step3_5_logging.rs`

Sample は出力先を `logs/step3-5/`、prefix を `wfide-step3-5`、保持数を 7 に設定する。

## Windows 11 検証項目

| 項目 | 結果 |
| --- | --- |
| `cargo build --lib` | ？ |
| `cargo run --example step3_5_logging` | ？ |
| Window 起動 | ？ |
| console に WFIDE 起動ログ出力 | ？ |
| `logs/step3-5/` にログファイル生成 | ？ |
| 日次 rotation 設定 | ？ |
| 最大保持数 7 の設定 | ？ |
| in-memory buffer 取得 | ？ |
| Consumer/Application から `wfide::tracing` 利用 | ？ |
| Linux | ？ |
| macOS | ？ |

実機確認前は ○ にしない。

## rotation 方針

v0.1.0 では日次 rotation + 最大保持ファイル数を採用する。これにより通常利用でログファイル数が無制限に増加することを防ぐ。

サイズベース rotation は Step 3.5 の必須条件にはしない。将来、長時間に大量ログを出す用途で必要性を確認した場合に追加検討する。

## 後続 Step との境界

Step 3.5 では Log Panel UI、検索、フィルタ UI、JSON 永続化、別 process Runtime からのログ転送は実装しない。

Step 4 は Dock Layout を扱い、Log Panel の実際の Viewer 接続は Panel 実装/API の後続 Step で行う。
