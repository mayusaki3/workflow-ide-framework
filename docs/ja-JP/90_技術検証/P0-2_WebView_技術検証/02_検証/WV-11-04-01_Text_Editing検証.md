<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260912-104800Z-WV15
lang: ja-JP
canonical_title: WV-11-04-01 Browser Surface Text Editing 検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-04-01 Browser Surface Text Editing 検証

# WV-11-04-01 Browser Surface Text Editing 検証

## 目的

CEF OSR Browser Surface の編集可能要素に対し、範囲選択および Clipboard を伴う基本編集操作を実行できることを確認する。

WV-11-04 で成立した Pointer / Keyboard / Text 入力経路を前提とし、本検証では編集操作の成立性を対象とする。

## 前提

- WV-11-04 により Browser Surface への Pointer / Keyboard / Text 入力が成立している。
- Browser Surface 内の編集可能要素へフォーカスできる。
- 正式 Surface API は本検証では定義しない。
- 右クリック Context Menu 固有操作は本検証の対象外とする。
- IME Composition は WV-11-04-02 で別途検証する。

## 検証順序

1. Keyboard による範囲選択
2. Pointer Drag による範囲選択
3. Copy
4. Paste
5. Cut
6. Select All

## EDIT-IT-SPEC-001 Keyboard 範囲選択
<!-- hldocs:sec_id=sec_f3n8q2v6m1ka -->

### 概要

Browser Surface 内の編集可能要素で Keyboard 操作により文字範囲を選択できることを確認する。

### 前提条件

- 編集可能要素が Keyboard 入力対象になっている。
- Keyboard 押下および解放を Browser へ転送できる。

### 検証内容

- 文字列を入力済みの編集可能要素で Shift と移動キーを組み合わせて範囲選択する。

### 期待結果

- Keyboard 操作に応じた文字範囲が Browser 側で選択状態になる。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_f3n8q2v6m1ka

## EDIT-IT-SPEC-002 Pointer Drag 範囲選択
<!-- hldocs:sec_id=sec_j7c1w9r4p5tx -->

### 概要

Browser Surface 内の編集可能要素で Pointer Drag により文字範囲を選択できることを確認する。

### 前提条件

- Pointer 座標変換、移動、Button 入力が成立している。

### 検証内容

- 編集可能要素の文字列上で Pointer Button を押下したまま移動し、任意範囲で解放する。

### 期待結果

- Pointer Drag に対応する文字範囲が Browser 側で選択状態になる。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_j7c1w9r4p5tx

## EDIT-IT-SPEC-003 Copy
<!-- hldocs:sec_id=sec_m2k6a8d1v4qs -->

### 概要

Browser Surface 内で選択した文字列を Clipboard へ Copy できることを確認する。

### 前提条件

- Browser 側で文字範囲を選択できる。

### 検証内容

- 選択済み文字列に対して Copy 操作を行う。

### 期待結果

- 選択文字列が OS Clipboard に反映される。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_m2k6a8d1v4qs

## EDIT-IT-SPEC-004 Paste
<!-- hldocs:sec_id=sec_q9b3t7n1c5wy -->

### 概要

OS Clipboard の文字列を Browser Surface 内の編集可能要素へ Paste できることを確認する。

### 前提条件

- OS Clipboard に文字列が格納されている。
- Browser Surface 内の編集可能要素へフォーカスできる。

### 検証内容

- 編集可能要素へ Paste 操作を行う。

### 期待結果

- Clipboard の文字列が Browser 側の編集可能要素へ挿入される。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_q9b3t7n1c5wy

## EDIT-IT-SPEC-005 Cut
<!-- hldocs:sec_id=sec_v1r5m8k2d7pa -->

### 概要

Browser Surface 内で選択した文字列を Cut できることを確認する。

### 前提条件

- Browser 側で文字範囲を選択できる。

### 検証内容

- 選択済み文字列に対して Cut 操作を行う。

### 期待結果

- 選択文字列が編集可能要素から削除される。
- 削除された文字列が OS Clipboard に反映される。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_v1r5m8k2d7pa

## EDIT-IT-SPEC-006 Select All
<!-- hldocs:sec_id=sec_c8x4p1m6r9bz -->

### 概要

Browser Surface 内の編集対象全体を Select All 操作で選択できることを確認する。

### 前提条件

- Browser Surface 内の編集可能要素が Keyboard 入力対象になっている。

### 検証内容

- 編集可能要素へ Select All 操作を行う。

### 期待結果

- 編集可能要素内の対象文字列全体が選択状態になる。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_c8x4p1m6r9bz

## 検証状態

| 動作対象 | Key Select | Drag Select | Copy | Paste | Cut | Select All |
| --- | --- | --- | --- | --- | --- | --- |
| Windows 実機 | ○ | ○ | ？ | ？ | ？ | ？ |
| Linux VM | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux X11 実機 | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux Wayland 実機 | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Apple Silicon | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Intel | ？ | ？ | ？ | ？ | ？ | ？ |

## 完了条件

利用可能な検証環境で EDIT-IT-SPEC-001〜006 を実施し、各動作対象の結果を `○` / `×` / `？` で記録できること。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-04-01 Browser Surface Text Editing 検証
