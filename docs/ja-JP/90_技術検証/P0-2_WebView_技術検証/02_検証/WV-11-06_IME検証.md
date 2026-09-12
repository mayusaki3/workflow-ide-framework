<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260912-104801Z-WV16
lang: ja-JP
canonical_title: WV-11-06 Browser Surface IME 検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-06 Browser Surface IME 検証

# WV-11-06 Browser Surface IME 検証

## 目的

CEF OSR Browser Surface の編集可能要素に対し、OS の IME を使用した Composition と確定入力を実現できることを確認する。

WV-11-04 の Keyboard / Text 入力成立性を前提とし、本検証では IME 固有の入力状態と表示位置同期を対象とする。

## 前提

- Browser Surface 内の編集可能要素へフォーカスできる。
- Browser Surface の表示矩形と Browser OSR 座標を対応付けられる。
- OS 固有の IME 差異は Framework 内部で吸収する方針とする。
- 正式 Surface API は本検証では定義しない。
- IME を利用できない検証環境は未検証 `？` とする。

## 検証順序

1. IME 入力開始
2. Composition 更新
3. Candidate / Composition 表示位置同期
4. Composition 確定
5. Composition キャンセル

## IME-IT-SPEC-001 IME 入力開始
<!-- hldocs:sec_id=sec_n4q8c2v7m1kt -->

### 概要

Browser Surface 内の編集可能要素を入力対象とした状態で OS IME を開始できることを確認する。

### 前提条件

- 編集可能要素へフォーカスできる。

### 検証内容

- OS IME を有効化し、編集可能要素へ IME 入力を開始する。

### 期待結果

- Browser Surface の編集可能要素を対象として IME Composition を開始できる。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_n4q8c2v7m1kt

## IME-IT-SPEC-002 Composition 更新
<!-- hldocs:sec_id=sec_b6m1r9p3x7da -->

### 概要

IME の未確定文字列および選択状態を Browser Surface の編集可能要素へ反映できることを確認する。

### 前提条件

- IME Composition を開始できる。

### 検証内容

- IME 入力中に未確定文字列を更新し、変換操作を行う。

### 期待結果

- Browser 側で未確定文字列の変化が反映される。
- Composition 内の選択状態を保持できる。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_b6m1r9p3x7da

## IME-IT-SPEC-003 Candidate / Composition 表示位置同期
<!-- hldocs:sec_id=sec_t5k2w8c1q6vz -->

### 概要

IME Candidate Window または Composition UI の表示基準位置を Browser 内の入力位置と同期できることを確認する。

### 前提条件

- Browser 内の入力位置または Composition Range に対応する座標を取得できる。
- Browser OSR 座標を Framework の表示座標へ対応付けられる。

### 検証内容

- 編集可能要素の異なる位置で IME Composition を行う。

### 期待結果

- IME Candidate Window または Composition UI が対象入力位置に対応する位置へ表示される。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_t5k2w8c1q6vz

## IME-IT-SPEC-004 Composition 確定
<!-- hldocs:sec_id=sec_h9p4m1d7r2cx -->

### 概要

IME Composition で確定した文字列を Browser Surface の編集可能要素へ反映できることを確認する。

### 前提条件

- IME Composition を更新できる。

### 検証内容

- IME 変換結果を確定する。

### 期待結果

- 確定文字列が Browser 側の編集可能要素へ反映される。
- Composition 状態が終了する。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_h9p4m1d7r2cx

## IME-IT-SPEC-005 Composition キャンセル
<!-- hldocs:sec_id=sec_w3c7n5a1m8qs -->

### 概要

進行中の IME Composition をキャンセルして Browser Surface の編集状態を正常に復帰できることを確認する。

### 前提条件

- IME Composition が進行中である。

### 検証内容

- 未確定文字列が存在する状態で Composition をキャンセルする。

### 期待結果

- 未確定文字列が確定文字列として残らない。
- Browser Surface の編集可能要素が通常の入力状態へ戻る。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_w3c7n5a1m8qs

## 検証状態

| 動作対象 | Start | Composition | Position | Commit | Cancel |
| --- | --- | --- | --- | --- | --- |
| Windows 実機 | ？ | ？ | ？ | ？ | ？ |
| Linux VM | ？ | ？ | ？ | ？ | ？ |
| Linux X11 実機 | ？ | ？ | ？ | ？ | ？ |
| Linux Wayland 実機 | ？ | ？ | ？ | ？ | ？ |
| macOS Apple Silicon | ？ | ？ | ？ | ？ | ？ |
| macOS Intel | ？ | ？ | ？ | ？ | ？ |

## 完了条件

利用可能な検証環境で IME-IT-SPEC-001〜005 を実施し、各動作対象の結果を `○` / `×` / `？` で記録できること。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-06 Browser Surface IME 検証
