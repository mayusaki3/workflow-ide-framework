[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > PoC-1c 親Window取得

# PoC-1c 親Window取得

## 目的

eframe 0.33 から winit::window::Window を取得可能か確認する。

## 成功条件

* Window Handle を取得できる
* Window Position を取得できる
* Window Size を取得できる

## 失敗条件

* eframe API から Window に到達できない

## 次工程

成功:
PoC-1d Child Window生成

失敗:
別Window管理方式を検討

---

[目次](../../../目次.md) > [技術検証目次](../../技術検証目次.md) > [P0-2 WebView 技術検証](../README.md) > PoC-1c 親Window取得
