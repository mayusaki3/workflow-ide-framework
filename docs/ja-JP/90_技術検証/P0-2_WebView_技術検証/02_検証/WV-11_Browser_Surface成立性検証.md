目次 > 技術検証 > P0-2_WebView_技術検証 > 02_検証 > WV-11_Browser_Surface成立性検証

# WV-11 Browser Surface成立性検証

## 目的

Windows を主対象として、Dock 上で Browser Surface を成立させる。
Linux は成立性を確認する。

## 検証項目

- WV-11-01 CEF OSR の Rust ライブラリ調査
- WV-11-02 CEF OSR 描画取得
- WV-11-03 egui Texture 転送
- WV-11-04 入力イベント転送
- WV-11-05 Windows 実装
- WV-11-06 Linux 成立性確認
- WV-11-07 API 整理

## 完了条件

- Windows 上で Browser Surface が Dock 内で動作する。
- OS 非依存 API の成立見込みを確認する。

---

目次 > 技術検証 > P0-2_WebView_技術検証 > 02_検証 > WV-11_Browser_Surface成立性検証