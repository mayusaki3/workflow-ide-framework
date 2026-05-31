<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260531-000003Z-WV27
lang: ja-JP
canonical_title: WV-01 WebView表示
document_type: test_result
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [WebView 検証結果](./WebView_検証結果.md) > WV-01 WebView表示

# WV-01 WebView表示

## 目的

単一 WebView の生成および表示が成立することを確認する。

## 前提

WV-00 により、Dock 埋め込み成立性確認は条件付き成功と判定済みである。

WV-01 では、wry を利用した WebView 生成と、eframe / egui との基本的な共存可否を確認する。

## 実施結果

### PoC-2a wry導入確認

#### 確認内容

- `Cargo.toml`
- `wry 0.53.5`
- `webview2-com`

#### 確認結果

Windows:

- `cargo build` 成功
- wry 導入成功
- webview2-com 導入成功
- eframe 0.33.0 と共存可能

#### 判断

依存関係競合は確認されなかった。

WV-01 を継続可能と判断する。

---

### PoC-2b build_as_child調査

#### 確認内容

- `WebViewBuilder`
- `build_as_child`

#### 確認結果

Windows:

- `build_as_child` 存在確認
- HWND は `HasWindowHandle` を実装していない

確認結果:

- HWND から `build_as_child` は利用不可
- Child Window HWND を直接親として利用できない

#### 判断

HWND 直結方式は採用しない。

wry が要求する Window Handle 抽象化を利用する方式へ移行する。

---

### PoC-2c WebViewBuilder調査

#### 確認内容

- `WebViewBuilder::new()`
- `with_url()`

#### 確認結果

Windows:

- `WebViewBuilder::new()` 成功
- `with_url("https://example.com")` 成功
- `cargo run` 成功

ログ:

```text
WV-01 create start
WV-01 builder created
WV-01 PoC-2c ready
```

確認結果:

- Builder 生成成功
- URL 設定成功
- 実行時異常なし

#### 判断

wry API は利用可能である。

WV-01 の前提条件は成立した。

次工程として、PoC-2d 実 WebView 生成確認へ進む。

---

### PoC-2d HasWindowHandle調査

#### 目的

`eframe` が提供する `HasWindowHandle` 実装を利用して WebView を生成可能か確認する。

#### 確認内容

- `build_as_child()`
- `build()`
- `eframe::Frame` 利用可否
- WebView 生成可否

#### 成功条件

- WebView 生成成功
- example.com 表示成功

#### 失敗条件

- `HasWindowHandle` を利用した WebView 生成不可

#### 確認結果

Windows:

- `build_as_child(frame)` 成功
- WebView 生成成功
- example.com 表示成功
- アプリケーション異常終了なし

ログ:

```text
WV-01 create start
WV-01 WebView create success
```

#### 判断

`eframe::Frame` を利用した WebView 生成は可能である。

HWND を利用しない方式で WebView 表示が成立した。

次工程として、PoC-2e Dock Panel 配置確認へ進む。

## 検証ケース結果

| 検証番号 | 結果 | 備考 |
|---|---|---|
| WV-01-01 | 成功 | WebView生成成功 |
| WV-01-02 | 成功 | example.com表示成功 |
| WV-01-03 | 成功 | アプリケーション異常終了なし |
| WV-01-04 | 成功 | `build_as_child(frame)` 成功 |

## 判定

成功。

以下を確認した。

- `build_as_child(frame)` 成功
- WebView 生成成功
- example.com 表示成功
- アプリケーション異常終了なし

## 課題

### 課題-01

wry 0.53.5 の `build_as_child()` は `HasWindowHandle` を要求する。

HWND は `HasWindowHandle` を実装していないため、PoC-1e で生成した Child Window HWND を直接利用できない。

### 課題-02

`build_as_child(frame)` は Root Window 上に WebView を生成する。

Dock Panel には配置されないため、Dock 移動・Dock レイアウト変更へ追従しない。

## 次工程

PoC-2e:

- Dock Panel 配置確認

WV-02:

- egui 共存確認

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > [WebView 検証結果](./WebView_検証結果.md) >WV-01 WebView表示
