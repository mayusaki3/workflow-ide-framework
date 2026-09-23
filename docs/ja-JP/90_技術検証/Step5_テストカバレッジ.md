# Step 5 テストカバレッジ管理

## 1. Coverageの意味
ここでは「コードカバレッジ」と「要求/GUIカバレッジ」を分ける。Rustのline coverageだけではIME、Window、Dock、drag、scroll、GPU/Browser等の成立性を評価できない。

## 2. 現在の要求カバレッジ
| Step | 自動test | Windows GUI | Linux VM | macOS |
|---|---|---|---|---|
| 5.5 Text Editor | ○ model基本 | ○ 主要操作/IME | ○ 主要表示・IME検証あり | ？ |
| 5.6 Log Viewer | ？ viewer GUI中心 | ○ 主要表示 | ○ 主要表示（horizontal scroll等一部？） | ？ |
| 5.7 Tree Viewer | ○ model基本 | ○ 基本表示/選択 | ？ | ？ |
| 5.8 Flow Editor | ○ model基本 | ○ 基本milestone | ？ | ？ |
| 5.9 Property Panel | ○ adapter基本 | ○ 基本UI、日本語表示/入力 | ？ | ？ |

注: ○は既存の明示的な確認範囲だけを示し、Step全機能100%を意味しない。

## 3. 未カバー/残件
- 5.6 DEBUG/TRACE色、Windows ERROR色、auto-scroll動的追従の明示確認。
- 5.7 Linux VM/macOS GUI。
- 5.8 Linux VM/macOSの新しいnavigation/connection UI。
- 5.9 editable Integer、Property→Flow双方向GUI統合、Linux VM/macOS。
- GUI eventをheadless unit testへ無理に移さない。
- Consumer acceptanceは未実施。

## 4. Code coverage baseline
未取得。Meridian依頼前に `cargo llvm-cov --workspace --all-targets --summary-only` の実測値を記録する。実測前に推定値を書かない。

## 5. 更新規則
機能追加時は、仕様→test case→自動testまたはmanual acceptance→結果→coverage matrixの順に追跡する。未確認を○へ変更する場合は実行環境と証跡を残す。
