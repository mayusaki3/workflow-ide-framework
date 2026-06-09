//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! WV-08-10
//!
//! 役割:
//! - gtk::init()、GTK Window生成、Root Fixed生成、Child Fixed生成、Child Fixed move/resize、GTK Label生成を実行する。
//! - wry::WebViewBuilder::build_gtk(&child_fixed) により WebKitGTK WebView を最小生成する。
//! - Root Fixed を GTK Window に追加し、Child Fixed を Root Fixed に追加する。
//! - Child Fixed を固定位置へ移動し、固定サイズを設定する。
//! - GTK Label を Child Fixed に追加する。
//! - GTK Window / Root Fixed / Child Fixed / WebView を static に保持する。
//! - show_all() 直後に上限付き GTKイベントflush を1回実行する。
//! - 継続的なGTKイベント処理、WebView追従move/resizeは実行しない。
//! - WebKitGTK WebView最小生成だけで応答なしが発生するか確認する。

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use std::time::{Duration, Instant};
use wry::{WebView, WebViewBuilder, WebViewBuilderExtUnix};

// 以下変更なし