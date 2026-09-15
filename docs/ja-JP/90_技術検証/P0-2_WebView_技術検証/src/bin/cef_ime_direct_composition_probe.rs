// WV-11-04-02 Direct CEF IME Composition Probe。
//
// 役割:
// - Windows native IME / WM_IME_* を介さず、CEF BrowserHost::ImeSetComposition を直接呼ぶ。
// - CEF 自身の OSR IME unit test と同等の replacement_range / selection_range を使用し、
//   Browser 側 compositionstart / compositionupdate が成立するかを切り分ける。
// - Browser input のクリック後、3種類の Composition 条件を時間差で自動送信し、
//   egui 側の追加操作に依存せず CEF OSR 側の Composition 受理そのものを検証する。
//
// 注意点:
// - 本ファイルは技術検証用であり、正式 Surface API ではない。
// - Browser input を1回クリックした後は操作不要。3方式を約1秒間隔で自動送信する。
// - Windows IME を有効にする必要はない。
// - IME-IT-SPEC-001/002/003 の合否は、この Probe 単独では確定しない。

mod probe {
    include!("cef_ime_native_bridge_probe.rs");

    const AUTO_STEP_INTERVAL: Duration = Duration::from_secs(1);

    #[derive(Clone, Copy, Debug)]
    enum DirectCompositionMode {
        CefUnitTestStyle,
        WindowsStyle,
        CollapsedSelection,
    }

    #[derive(Clone)]
    struct DirectCompositionTask {
        browser: Browser,
        mode: DirectCompositionMode,
    }

    wrap_task! {
        struct DirectCompositionTaskBuilder { task: DirectCompositionTask, }
        impl Task {
            fn execute(&self) {
                let Some(host) = self.task.browser.host() else { return; };
                host.set_focus(true.into());

                let source = "にほん";
                let text = CefString::from(source);
                let text_len = source.encode_utf16().count() as u32;
                let underline = CompositionUnderline {
                    range: Range { from: 0, to: text_len },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: 0,
                    ..Default::default()
                };
                let underlines = [underline];

                match self.task.mode {
                    DirectCompositionMode::CefUnitTestStyle => {
                        // CEF tests/ceftests/os_rendering_unittest.cc と同じく、
                        // replacement_range / selection_range の双方を 0..text_len にする。
                        let replacement = Range { from: 0, to: text_len };
                        let selection = Range { from: 0, to: text_len };
                        host.ime_set_composition(
                            Some(&text),
                            Some(&underlines),
                            Some(&replacement),
                            Some(&selection),
                        );
                        println!(
                            "Direct CEF IME AUTO 1/3: unit-test style text={:?} replacement={}..{} selection={}..{}",
                            source, replacement.from, replacement.to, selection.from, selection.to
                        );
                    }
                    DirectCompositionMode::WindowsStyle => {
                        // cefclient Windows OSR と同様に replacement_range を無効扱い(None)とする。
                        let selection = Range { from: 0, to: text_len };
                        host.ime_set_composition(
                            Some(&text),
                            Some(&underlines),
                            None,
                            Some(&selection),
                        );
                        println!(
                            "Direct CEF IME AUTO 2/3: Windows style text={:?} replacement=None selection={}..{}",
                            source, selection.from, selection.to
                        );
                    }
                    DirectCompositionMode::CollapsedSelection => {
                        // selection_range 解釈の影響を切り分けるため、caret のみ 0..0 を指定する。
                        let selection = Range { from: 0, to: 0 };
                        host.ime_set_composition(
                            Some(&text),
                            Some(&underlines),
                            None,
                            Some(&selection),
                        );
                        println!(
                            "Direct CEF IME AUTO 3/3: collapsed selection text={:?} replacement=None selection=0..0",
                            source
                        );
                    }
                }
            }
        }
    }

    impl DirectCompositionTaskBuilder {
        fn build(browser: Browser, mode: DirectCompositionMode) -> Task {
            Self::new(DirectCompositionTask { browser, mode })
        }
    }

    fn request_direct_composition(runtime: &ProbeRuntime, mode: DirectCompositionMode) -> bool {
        let Some(browser) = runtime
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned())
        else {
            return false;
        };
        let mut task = DirectCompositionTaskBuilder::build(browser, mode);
        post_task(ThreadId::UI, Some(&mut task)) == 1
    }

    struct DirectProbeApp {
        runtime: ProbeRuntime,
        texture: Option<egui::TextureHandle>,
        applied_generation: u64,
        current_click: Option<ClickTransfer>,
        auto_started_at: Option<Instant>,
        auto_step: usize,
        last_status: String,
    }

    impl DirectProbeApp {
        fn new(runtime: ProbeRuntime) -> Self {
            Self {
                runtime,
                texture: None,
                applied_generation: 0,
                current_click: None,
                auto_started_at: None,
                auto_step: 0,
                last_status: "Click Browser input once to start automatic test".to_string(),
            }
        }

        fn update_texture(&mut self, ctx: &egui::Context) {
            let snapshot = {
                let Ok(state) = self.runtime.state.lock() else { return; };
                if state.generation == self.applied_generation
                    || state.width <= 0
                    || state.height <= 0
                    || state.rgba.is_empty()
                {
                    None
                } else {
                    Some((
                        state.generation,
                        state.width as usize,
                        state.height as usize,
                        state.rgba.clone(),
                    ))
                }
            };
            let Some((generation, width, height, rgba)) = snapshot else { return; };
            let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
            match self.texture.as_mut() {
                Some(texture) if texture.size() == [width, height] => {
                    texture.set(image, egui::TextureOptions::LINEAR);
                }
                _ => {
                    self.texture = Some(ctx.load_texture(
                        "wv11_04_02_direct_cef_ime",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            self.applied_generation = generation;
        }

        /// Browser input クリック後に3方式を約1秒間隔で自動送信する。
        ///
        /// # 戻り値
        /// - なし。各送信結果は `last_status` とコンソールログへ記録する。
        fn advance_auto_test(&mut self) {
            let Some(started_at) = self.auto_started_at else { return; };
            if self.auto_step >= 3 {
                return;
            }

            let required_elapsed = AUTO_STEP_INTERVAL * ((self.auto_step + 1) as u32);
            if started_at.elapsed() < required_elapsed {
                return;
            }

            let (mode, label) = match self.auto_step {
                0 => (DirectCompositionMode::CefUnitTestStyle, "1/3 CEF unit-test style"),
                1 => (DirectCompositionMode::WindowsStyle, "2/3 Windows style"),
                2 => (DirectCompositionMode::CollapsedSelection, "3/3 selection 0..0"),
                _ => return,
            };

            if request_direct_composition(&self.runtime, mode) {
                self.auto_step += 1;
                self.last_status = format!("sent automatically: {label}");
                if self.auto_step == 3 {
                    self.last_status.push_str("; automatic sequence complete");
                }
            } else {
                self.last_status = format!("failed to post automatically: {label}");
            }
        }
    }

    impl eframe::App for DirectProbeApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.update_texture(ctx);
            self.advance_auto_test();

            let (generation, callbacks, bounds, selected_range) = self
                .runtime
                .state
                .lock()
                .map(|state| {
                    (
                        state.generation,
                        state.ime_range_callbacks,
                        state.character_bounds.len(),
                        state.selected_range.clone(),
                    )
                })
                .unwrap_or((0, 0, 0, None));

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("WV-11-04-02 Direct CEF IME Composition - automatic");
                    ui.separator();
                    ui.label(format!("Paint: {generation}"));
                    ui.separator();
                    ui.label(format!("CEF range callbacks: {callbacks}"));
                    ui.separator();
                    ui.label(format!("Bounds: {bounds}"));
                    ui.separator();
                    ui.label(format!("Selected: {:?}", selected_range));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Auto step: {}/3", self.auto_step));
                    ui.separator();
                    ui.label(format!("Status: {}", self.last_status));
                });
            });

            self.current_click = None;
            egui::CentralPanel::default().show(ctx, |ui| {
                let available = ui.available_size();
                if let Some(texture) = self.texture.as_ref() {
                    let response = ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(available)
                            .sense(egui::Sense::click()),
                    );
                    if response.clicked() && self.auto_started_at.is_none() {
                        if let Some(pointer) = response.interact_pointer_pos() {
                            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                                self.current_click = Some(ClickTransfer { x, y });
                                self.auto_started_at = Some(Instant::now());
                                self.auto_step = 0;
                                self.last_status =
                                    "Browser input clicked; automatic sequence starts in 1 second"
                                        .to_string();
                                println!(
                                    "Direct CEF IME automatic sequence armed after Browser click: x={} y={}",
                                    x, y
                                );
                            }
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| ui.label("Waiting for CEF OSR Paint..."));
                }
            });

            if let Some(click) = self.current_click {
                let _ = self.runtime.request_click(click);
            }
            ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
        }
    }

    pub fn run() -> eframe::Result<()> {
        let cli_args: Vec<String> = std::env::args().collect();
        if is_cef_subprocess(&cli_args) {
            std::process::exit(run_subprocess());
        }

        println!("WV-11-04-02 direct CEF IME composition automatic probe start");
        println!("Click Browser input once. Three direct CEF composition modes are then sent automatically.");

        let runtime = match ProbeRuntime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("direct CEF IME probe failed: {error}");
                std::process::exit(1);
            }
        };

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 Direct CEF IME Composition Probe")
                .with_inner_size([1100.0, 760.0]),
            ..Default::default()
        };

        eframe::run_native(
            "WV-11-04-02 Direct CEF IME Composition Probe",
            native_options,
            Box::new(move |_cc| Ok(Box::new(DirectProbeApp::new(runtime)))),
        )
    }
}

fn main() -> eframe::Result<()> {
    probe::run()
}
