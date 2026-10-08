<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261008-kairi-workspace-public-if
lang: ja-JP
canonical_title: Workspace Public I/F 設計
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > 仕様 > Workspace > Public I/F設計

# Workspace Public I/F 設計

## 1. 目的・境界

[Workspace仕様](03_Workspace.md)および[Workspaceテスト仕様](../04_テスト仕様/03_Workspace_テスト仕様.md)を満たす、利用アプリケーション向けの公開操作境界を定義する。以下は実装前の論理I/F設計であり、Rustの具体的な型名・method signature・error enumは実装時のAPI Referenceで確定する。

WorkspaceはPanelの所有者ではない。WorkspaceとContainerは配置状態を管理し、Panel instanceの生成・破棄は既存のPanel管理機構に委ねる。

## 2. 識別と登録

- Workspace IDはstable opaque ID。表示名とは分離する。
- `framework.workspace.default` は予約IDであり、Application初期化時に必ず存在する。
- ApplicationはWorkspaceの追加登録、一覧取得、選択、登録解除を行える。
- Default Workspaceの登録解除・予約IDの再登録は拒否する。
- Workspace登録時にはDefault Containerを一つ自動生成する。
- Container IDはWorkspace内で一意とし、別WorkspaceのContainerを参照する操作は拒否する。
- Workspace/Container/Panelの未登録参照は暗黙に生成せず、呼び出し側へ失敗を返す。

## 3. 公開操作の論理分類

| 分類 | 操作 | 主な入力 | 結果 |
|---|---|---|---|
| Workspace | register / unregister / list / select | Workspace ID、表示名 | 成否・対象情報 |
| Container | register / list / get default | Workspace ID、Container ID | 成否・対象情報 |
| Layout | set workspace layout / set container layout | Workspace ID、Container ID、Layout定義 | 成否 |
| Panel配置 | place / move / hide / show | Workspace ID、Panel instance ID、配置先 | 成否・配置状態 |
| Layout状態 | query / save / restore / reset | Workspace ID、Project session | 成否・状態 |
| Dialog表示 | attach / detach view | Workspace ID、表示先 | 成否 |

この表の操作名は論理上の名称であり、具体的なRust公開関数名を規定しない。

## 4. Panel配置と再表示先の決定

同一Panel instanceの同一Workspace内二重配置は拒否する。異なるWorkspaceで同一instanceを参照することは許可し、配置状態は独立させる。

Panel再表示時の配置先は次の順序で決定する。

1. 再表示操作で明示された有効なContainer。
2. 対象Workspaceで非表示になる直前の有効な配置先。Floatingであれば保存された位置・サイズで専用Floating Containerを再生成する。
3. 以前の配置先が無効・不明の場合は対象WorkspaceのDefault Container。

明示指定が無効な場合はエラーとし、指定を無視してDefault Containerへ暗黙に移動させない。別Workspaceの配置履歴は参照しない。

## 5. Drag / Floating

- 通常Container間のPanel移動を許可する。ドラッグ操作はPanel instanceの所有権を移動しない。
- 通常Containerの外へFloating化した場合はPanelごとに専用Floating Containerを生成する。
- Floating Containerは単一Panel専用で、**別Panelのドラッグ＆ドロップによる追加、タブ化、分割、置換を受け付けない**。
- Floating Containerから通常ContainerへPanelを移動した場合、空のFloating Containerを破棄する。
- Floating Panelを非表示にした場合、Containerを破棄して位置・サイズを対象Workspaceの配置状態に残す。
- Floating位置・サイズはProject内User Layoutへ保存する。

## 6. Layout永続化とリセット

- WorkspaceごとのUser LayoutをProject内のUser設定として保存する。
- 復元の優先順位は、保存済みUser Layout、Application既定Layout、Default Containerのみの順とする。
- 保存済みUser LayoutはApplication既定Layout更新時にも暗黙にマージしない。
- resetは必ず対象Workspace IDを受け取り、そのWorkspaceのUser Layoutのみ破棄して現在のApplication既定Layoutから再構成する。
- 非表示Workspaceを含む一括・暗黙のリセットは行わない。
- Layout変更によるProject dirty stateはKairi管理状態として扱い、既存Project Save/Close規則に従う。

## 7. Dialogとlifetime

Workspaceの生成・破棄は所有者の責務とし、Main WindowまたはDialogへの表示開始・終了と分離する。Dialogの終了だけでWorkspaceやPanel instanceを破棄しない。

v0.1.0のProject dialogでは共通Workspace機構を内部利用して検証するが、汎用Dialog公開I/Fは別途設計する。

## 8. エラー契約

少なくとも次を区別できる結果を返す。

- 重複ID・予約ID使用
- Workspace/Container/Panel未登録
- 同一Workspace内Panel重複配置
- ContainerのWorkspace不一致
- Default Workspaceの削除要求
- Floating ContainerへのPanelドロップ拒否
- Layoutの不正な参照または構成

失敗時に未登録の対象を暗黙に生成しない。失敗操作によって既存の有効なPanel配置を破壊しない。

## 9. 仕様テストとの対応

- WS-001～WS-008：識別、登録、Default Container
- WS-009～WS-019：Layoutと配置先検証
- WS-020～WS-027：Panel instanceの共有とWorkspace切替
- WS-028～WS-036：永続化、優先順位、対象指定リセット
- WS-037～WS-041：Dialogの共通機構とlifetime
- WS-042～WS-047：Floating Containerと位置・サイズ復帰

本書は仕様テストの期待結果を実装都合で変更する根拠にはしない。実装後のカバレッジ補完テストは別管理とする。

---

[目次](../目次.md) > 仕様 > Workspace > Public I/F設計
