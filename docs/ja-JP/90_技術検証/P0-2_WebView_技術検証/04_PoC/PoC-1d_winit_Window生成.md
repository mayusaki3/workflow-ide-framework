<!--
HLDocS:LLM-MANAGED
doc_id: doc-20260530-000007Z-WV26
lang: ja-JP
canonical_title: PoC-1d winit Window生成
document_type: tech_note
canonical_document: true
-->

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > PoC-1d winit Window生成

# PoC-1d winit Window生成

## 目的

winit を直接利用して Window を生成可能か確認する。

本PoCは、WV-00 Dock埋め込み成立性確認における案B「egui_dock + Child Window + wry」へ進むための前提検証とする。

## 背景

P0-2 WebView 技術検証では、WebView 表示、egui coexist、Window coexist、Focus 切替を確認対象としている。

WV-00では、以下の方式を判定対象としている。

* 案A: egui_dock + wry Native View
* 案B: egui_dock + Child Window + wry
* 案C: egui_dock + HTML Renderer

既に以下は不採用と判定済みである。

* ViewportInfo依存
* FrameInfo依存
* CreationContext依存
* eframeからWindow取得方式

そのため、本PoCでは eframe 経由ではなく、winit 直接利用による Window 生成を確認する。

## 要件定義

### REQ-P0-2-POC-1D-001 winit直接利用

PoC実装は、eframe から Window を取得する方式に依存せず、winit の EventLoop を直接利用する。

### REQ-P0-2-POC-1D-002 複数Window生成

PoC実装は、主Windowと追加Windowを生成する。

### REQ-P0-2-POC-1D-003 EventLoop競合確認

PoC実装は、単一の winit EventLoop 上で複数Windowを扱い、起動時に EventLoop 競合でクラッシュしないことを確認する。

### REQ-P0-2-POC-1D-004 次工程への入力

PoC結果は、次工程 PoC-1e Child Window生成 の前提情報として扱えること。

## 設計

### 実装方針

* winit のみを直接利用する。
* ApplicationHandler 方式で EventLoop を実行する。
* resumed 時に主Windowと追加Windowを生成する。
* WindowEvent::CloseRequested を受けたら EventLoop を終了する。
* 本PoCでは WebView を生成しない。
* 本PoCでは Child Window 化しない。

### 実装場所

```text
tools/poc/p0-2-webview/poc-1d-winit-window/
```

### ファイル構成

```text
tools/poc/p0-2-webview/poc-1d-winit-window/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

### 非対象

* WebView表示
* wry利用
* Dock Panel追従
* OSネイティブの親子Window化
* egui_dock統合

これらは後続PoCで扱う。

## 検証ケース

### TC-P0-2-POC-1D-001 Windows起動確認

#### 目的

Windows上で winit Window を生成できることを確認する。

#### 手順

1. Windows環境で `tools/poc/p0-2-webview/poc-1d-winit-window` を開く。
2. `cargo run` を実行する。
3. 主Windowと追加Windowが表示されることを確認する。
4. Windowを閉じる。

#### 期待結果

* 主Windowが表示される。
* 追加Windowが表示される。
* Window生成時にクラッシュしない。
* Windowを閉じるとアプリケーションが終了する。

### TC-P0-2-POC-1D-002 Ubuntu Desktop起動確認

#### 目的

Ubuntu Desktop上で winit Window を生成できることを確認する。

#### 手順

1. Ubuntu Desktop環境で `tools/poc/p0-2-webview/poc-1d-winit-window` を開く。
2. `cargo run` を実行する。
3. 主Windowと追加Windowが表示されることを確認する。
4. Windowを閉じる。

#### 期待結果

* 主Windowが表示される。
* 追加Windowが表示される。
* Window生成時にクラッシュしない。
* EventLoop競合で起動失敗しない。

### TC-P0-2-POC-1D-003 EventLoop終了確認

#### 目的

WindowのCloseRequestedにより EventLoop を終了できることを確認する。

#### 手順

1. `cargo run` を実行する。
2. 表示されたWindowのいずれかを閉じる。
3. プロセスが終了することを確認する。

#### 期待結果

* CloseRequested により EventLoop が終了する。
* プロセスが残留しない。

## 成功条件

* winit の EventLoop を直接起動できる。
* 主Windowと追加Windowを同時に表示できる。
* Windows で表示できる。
* Ubuntu Desktop で表示できる。
* Window生成時にクラッシュしない。
* EventLoop競合で起動失敗しない。

## 失敗条件

* Window生成時にクラッシュする。
* EventLoop競合で起動できない。
* 主Windowまたは追加Windowのいずれかが表示されない。
* Windowを閉じてもプロセスが残留する。

## 実装

PoC-1d の実装を以下に追加した。

```text
tools/poc/p0-2-webview/poc-1d-winit-window/
```

実装は winit 直接利用のみを対象とし、WebView、wry、Child Window 化、Dock追従は含めない。

## 次工程

成功:

* PoC-1e Child Window生成

失敗:

* Window生成方式再検討

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > PoC-1d winit Window生成
