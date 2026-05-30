# PoC-1d winit Window生成

## 目的

winit を利用して追加 Window を生成可能か確認する。

## 成功条件

* eframe Window と同時に表示できる
* Windows で表示できる
* Ubuntu Desktop で表示できる

## 失敗条件

* Window生成時にクラッシュする
* EventLoop競合で起動できない

## 次工程

成功:
PoC-1e Child Window化

失敗:
Window生成方式再検討
