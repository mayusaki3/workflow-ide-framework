# REG-002 Framework管理Resource Registry 実装前テスト仕様（草案）

対象: REG-001。実装前に期待結果を固定する。未実装項目を合格扱いしない。

| ID | 操作・入力 | 期待結果 |
|---|---|---|
| REG-CORE-001 | 同一Scopeと正規化パスを2回登録 | 同一Resource IDが返る |
| REG-CORE-002 | 異なるScopeに同じ相対パスを登録 | 別Resource IDになる |
| REG-CORE-003 | 参照先を改名・移動 | Resource IDが変化しない |
| REG-CORE-004 | 同一Resource IDの重複登録を試行 | 一意性が維持される |
| REG-LIFE-001 | New→登録→Save→Close→Open | 同一IDと参照先を復元 |
| REG-LIFE-002 | Save As | ID維持、移行先に正しく保存 |
| REG-LIFE-003 | 別Projectへ切替 | 前ProjectのRegistryが混入しない |
| REG-LIFE-004 | Registry変更後Save失敗 | dirtyが維持される |
| REG-LIFE-005 | 保存データに同一Resource IDが複数存在 | 復元をエラーにし、黙って統合しない |
| REG-LIFE-006 | 保存データに正規化後同一参照が異なるIDで存在 | 復元をエラーにし、黙って統合しない |
| REG-LIFE-007 | 別Projectを開く際のRegistry復元が失敗 | 開く前のProject状態を保持する |
| REG-DEL-001 | 未参照ファイルをユーザーが削除 | 実体とRegistryを整合して更新 |
| REG-DEL-002 | 参照数3のファイルを削除要求 | 参照警告と続行確認を表示 |
| REG-DEL-003 | 参照数不明で削除要求 | 未参照扱いにせず不明と表示 |
| REG-DEL-004 | 削除確認をキャンセル | ファイル、Registry、アプリデータ不変 |
| REG-DEL-005 | 複数論理項目が同一ファイルを参照 | 片方の論理項目削除で共有ファイルを暗黙削除しない |
| REG-DEL-006 | アプリが論理削除を拒否 | 削除成功扱いにしない |
| REG-DEL-007 | 実ファイル削除中に失敗 | ジャーナル／復旧情報を残し不整合を通知 |
| REG-DEL-008 | 外部操作で登録ファイルが消失 | MissingとしてIDと参照先を保持 |
| REG-I18N-001 | display_name_keyを設定 | 現在Localeの文字列を取得 |
| REG-I18N-002 | 翻訳キー未登録 | 既存localizationのフォールバック動作に従う |
| REG-I18N-003 | help/{locale}/guide.mdをja-JPで展開 | help/ja-JP/guide.md |
| REG-I18N-004 | Locale切替 | Resource IDと保存テンプレート不変 |
| REG-I18N-005 | 指定Localeのファイル欠落 | 既定Localeへフォールバック |
| REG-I18N-006 | 既定Localeのファイルも欠落 | 未解決を返す |
| REG-I18N-007 | 不正Locale・親ディレクトリ遷移 | 拒否する |
| REG-I18N-008 | 多言語項目をSave/Open | 言語キーとテンプレートを復元 |
| REG-PANEL-001 | Save→Close→Open後、登録Resource IDをProject Panelで参照 | Framework復元Registryで解決できる |
| REG-PANEL-002 | Registry未登録の論理ノードを表示 | 正常表示 |
| REG-PANEL-003 | 登録Resource IDを複数ノードに表示 | 同一Resource ID、別Node ID |
| REG-PANEL-004 | 2引数Project Panel API | Applicationのビルダーで2引数APIが利用できる |
| REG-PANEL-005 | 存在しないResource IDをRegisteredResourceで参照 | 診断可能なエラー |
| REG-PANEL-006 | Registry登録・参照先変更後にProject Panelを更新 | 変更時のみ再構築要求、変更なしの場合は要求しない |

## アプリ向けRegistry操作API（REG-APP：実装前テスト）

FrameworkHost内のProjectControllerをアプリが直接所有・ロックしない。アプリへ渡すHandleは要求の送信口とし、Frameworkが自身のProjectControllerに適用する。読み取り結果・エラーを呼び出し側が取得できること。

| ID | 操作・入力 | 期待結果 |
|---|---|---|
| REG-APP-001 | アプリからResourceReference登録要求 | Framework所有Registryへ登録され、生成IDを取得できる |
| REG-APP-002 | 同一参照を再登録 | 同一ID、dirty増加なし、Project Panel再通知なし |
| REG-APP-003 | Resource IDで参照を取得 | 現在Projectの参照先を取得できる |
| REG-APP-004 | Resource IDで参照先更新 | ID維持、Framework dirty、Project Panel更新要求 |
| REG-APP-005 | Project未Openで登録・参照・更新 | 明示的なNoProjectエラー、状態不変 |
| REG-APP-006 | Project Aで発行した要求をProject Bへ切替後に処理 | Aの要求をBへ誤適用しない |
| REG-APP-007 | 更新対象不在・参照先衝突 | 明示的エラー、Registry不変 |
| REG-APP-008 | Save直前にアプリから登録要求 | 要求の適用順序を保証し、Saveで登録内容を永続化 |
| REG-APP-009 | アプリのProject Panel providerがFramework Registryを参照 | アプリ側Registry複製を要求しない |
| REG-APP-010 | Handle複製と複数要求 | 要求の順序・結果対応が保持される |

注: 同期呼び出し／キュー処理のいずれを採用する場合も、登録IDの取得タイミング、UIスレッドの再入、Project切替境界を明文化してから実装する。

## 検証条件
- 本実装前の仕様テストと、実装後のカバレッジテストを区別する。
- Windows/Linux/macOSを対象とし、未検証環境は未検証と明記する。
- `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --all-features`を実行する。
- 削除確認UI、キャンセル、部分失敗、復旧はモデル試験に加えGUI／統合試験で検証する。
