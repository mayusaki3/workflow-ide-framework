<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260911-120000Z-WV13
lang: ja-JP
canonical_title: WV-11-03 Browser Surface Texture 転送検証
document_type: testspec
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-03 Browser Surface Texture 転送検証

# WV-11-03 Browser Surface Texture 転送検証

## 目的

CEF OSR の描画結果を egui Texture として扱い、Native Window を使用せず Dock Panel 内へ継続表示できることを確認する。

WV-11-02 で確認した CEF OSR の Paint / Buffer / Update を前提とし、Browser Surface の描画出力を egui / Dock 側へ接続する部分のみを対象とする。

## 前提

- WV-11-02 の対象環境で CEF OSR の描画バッファ取得が成立している。
- CEF OSR の Paint バッファは BGRA 画素列として扱う。
- egui Texture へ渡す境界では RGBA8 へ正規化する。
- 入力イベント転送は WV-11-04 で検証し、本検証では対象外とする。
- 正式 Surface API は本検証では定義しない。

## 検証順序

1. BGRA から RGBA8 への画素形式正規化
2. egui Texture 生成
3. Paint に応じた Texture 更新
4. Dock Panel 内表示
5. Dock サイズ変更への追従
6. Browser 画面変化の継続反映

## TEX-IT-SPEC-001 Texture 画素形式正規化
<!-- hldocs:sec_id=sec_3n8qk2v5u7am -->

### 概要

CEF OSR の BGRA 画素バッファを egui Texture 入力用 RGBA8 画素列へ変換できることを確認する。

### 前提条件

- Paint コールバックから描画サイズと BGRA 画素バッファを取得できている。

### 検証内容

- BGRA の各画素について B / R チャンネルを入れ替え、RGBA8 画素列を生成する。
- 変換後も描画サイズに対応するバイト数を保持する。

### 期待結果

- RGBA8 画素列を生成できる。
- 変換後バッファ長が `width × height × 4` と一致する。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_3n8qk2v5u7am

## TEX-IT-SPEC-002 egui Texture 生成
<!-- hldocs:sec_id=sec_r8m4t2x9c6kp -->

### 概要

正規化済み RGBA8 バッファから egui Texture を生成できることを確認する。

### 前提条件

- TEX-IT-SPEC-001 により RGBA8 画素列を生成できている。

### 検証内容

- 描画サイズと RGBA8 画素列から egui の画像データを構成する。
- egui Context に Texture を登録する。

### 期待結果

- Texture 登録に成功する。
- Texture の論理サイズが描画バッファの幅・高さと一致する。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_r8m4t2x9c6kp

## TEX-IT-SPEC-003 Paint 連動 Texture 更新
<!-- hldocs:sec_id=sec_q6f1w9n3z8bd -->

### 概要

CEF OSR の新しい Paint に応じて既存 egui Texture を更新できることを確認する。

### 前提条件

- egui Texture を生成できている。
- Browser の画面変化に応じた新しい Paint を取得できる。

### 検証内容

- 初回 Paint で生成した Texture に対し、後続 Paint の RGBA8 画素列を反映する。

### 期待結果

- 同一 Texture を新しい描画内容へ更新できる。
- 更新後 Texture に後続 Paint の内容が反映される。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_q6f1w9n3z8bd

## TEX-IT-SPEC-004 Dock Panel 内表示
<!-- hldocs:sec_id=sec_h5v2c8m7p1rs -->

### 概要

egui Texture とした Browser Surface を Dock Panel 内へ描画できることを確認する。

### 前提条件

- egui Texture を生成・更新できる。

### 検証内容

- Texture を egui_dock の Panel 内容として描画する。

### 期待結果

- Browser の描画内容が Dock Panel 内へ表示される。
- Browser 用の独立 Native Window を表示しない。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_h5v2c8m7p1rs

## TEX-IT-SPEC-005 Dock サイズ同期
<!-- hldocs:sec_id=sec_b4j7k1s9d3wx -->

### 概要

Dock Panel の表示領域変更へ Browser OSR 描画領域と Texture サイズが追従できることを確認する。

### 前提条件

- Browser Surface が Dock Panel 内へ表示されている。

### 検証内容

- Dock Panel のサイズを変更する。
- 変更後サイズを Browser OSR 描画領域へ反映し、新しい Paint を Texture へ反映する。

### 期待結果

- Browser OSR 描画領域が変更後サイズへ更新される。
- Texture 表示が変更後サイズへ追従する。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_b4j7k1s9d3wx

## TEX-IT-SPEC-006 Dock 表示継続更新
<!-- hldocs:sec_id=sec_m9a2e6r4t7yc -->

### 概要

Browser の画面変化に応じた継続 Paint を Dock Panel 内表示へ反映できることを確認する。

### 前提条件

- Browser Surface が Dock Panel 内へ表示されている。
- Paint 連動 Texture 更新が成立している。

### 検証内容

- Browser 側の表示内容を複数回変化させる。
- それぞれの Paint を Texture 更新として Dock Panel 表示へ反映する。

### 期待結果

- 複数回の Browser 描画変化が Dock Panel 内へ継続反映される。
- 更新中に Browser 用の独立 Native Window を必要としない。

### 参照仕様

- doc-20260524-009601Z-R8M7#sec_m9a2e6r4t7yc

## 検証状態

| 動作対象 | Pixel | Texture | Update | Dock | Resize | Continuous |
| --- | --- | --- | --- | --- | --- | --- |
| Windows 実機 | ○ | ○ | ○ | ？ | ？ | ？ |
| Linux VM | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux X11 実機 | ？ | ？ | ？ | ？ | ？ | ？ |
| Linux Wayland 実機 | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Apple Silicon | ？ | ？ | ？ | ？ | ？ | ？ |
| macOS Intel | ？ | ？ | ？ | ？ | ？ | ？ |

## 完了条件

利用可能な検証環境で TEX-IT-SPEC-001〜006 を実施し、各動作対象の結果を `○` / `×` / `？` で記録できること。

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [検証目次](検証目次.md) > WV-11-03 Browser Surface Texture 転送検証
