// WV-11-04-02 CEF OSR parent HWND IME Probe。
//
// 役割:
// - eframe/winit の実 HWND を取得した後に CEF を初期化する。
// - `WindowInfo::set_as_windowless(parent)` でその HWND を CEF OSR Browser の親として渡す。
// - 既存 native IME -> CEF composition Bridge と比較し、parent HWND の有無が
//   Browser 側 compositionstart / compositionupdate 成立性へ影響するか切り分ける。
//
// 注意点:
// - 本ファイルは技術検証用であり、正式 Surface API ではない。
// - Composition 転送は既存 Bridge Probe と同じ単純な GCS_COMPSTR 転送を使用する。
//   直前の詳細 Bridge でも Browser 側 Composition が成立しなかったため、今回は
//   parent HWND の有無だけを主な変数として確認する。
// - IME-IT-SPEC-001/002/003 の合否は、この Probe 単独では確定しない。

mod probe {
    include!("cef_ime_native_bridge_probe.rs");

    #[derive(Clone)]
    struct ParentBrowserProcessHandler {
        state: Arc<Mutex<ProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        browser_create_failed: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
        parent_window: sys::HWND,
    }

    wrap_browser_process_handler! {
        struct ParentBrowserProcessHandlerBuilder {
            handler: ParentBrowserProcessHandler,
        }

        impl BrowserProcessHandler {
            fn on_context_initialized(&self) {
                let mut client = ProbeClientBuilder::build(
                    self.handler.state.clone(),
                    self.handler.browser.clone(),
                    self.handler.browser_created.clone(),
                    self.handler.closed.clone(),
                );

                // CEF cefclient の Windows OSR と同様に、実 native HWND を
                // windowless Browser の parent として関連付ける。
                let window_info = WindowInfo::default()
                    .set_as_windowless(self.handler.parent_window);
                let browser_settings = BrowserSettings {
                    windowless_frame_rate: 30,
                    ..Default::default()
                };
                let url = CefString::from(TEST_URL);
                let accepted = browser_host_create_browser(
                    Some(&window_info),
                    Some(&mut client),
                    Some(&url),
                    Some(&browser_settings),
                    None,
                    None,
                );
                if accepted != 1 {
                    self.handler
                        .browser_create_failed
                        .store(true, Ordering::Release);
                }
            }
        }
    }

    impl ParentBrowserProcessHandlerBuilder {
        fn build(handler: ParentBrowserProcessHandler) -> BrowserProcessHandler {
            Self::new(handler)
        }
    }

    #[derive(Clone)]
    struct ParentCefApp {
        browser_process_handler: BrowserProcessHandler,
    }

    wrap_app! {
        struct ParentCefAppBuilder {
            handler: ParentCefApp,
        }

        impl App {
            fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
                Some(self.handler.browser_process_handler.clone())
            }
        }
    }

    impl ParentCefAppBuilder {
        fn build(handler: ParentBrowserProcessHandler) -> App {
            Self::new(ParentCefApp {
                browser_process_handler: ParentBrowserProcessHandlerBuilder::build(handler),
            })
        }
    }

    struct ParentProbeRuntime {
        state: Arc<Mutex<ProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        closed: Arc<AtomicBool>,
    }

    impl ParentProbeRuntime {
        /// eframe HWND を親として CEF OSR Browser を初期化する。
        ///
        /// # 引数
        /// - `parent_hwnd`: eframe/winit が作成した Win32 HWND。
        ///
        /// # 戻り値
        /// - 初期化済み runtime。失敗時は理由を文字列で返す。
        ///
        /// @hldocs.ref doc-20260912-104801Z-WV16#sec_n4q8c2v7m1kt
        fn new(parent_hwnd: HWND) -> Result<Self, String> {
            let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
            let args = args::Args::new();
            let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
            if process_result >= 0 {
                return Err(format!(
                    "Unexpected subprocess result in browser process: {process_result}"
                ));
            }

            let settings = Settings {
                windowless_rendering_enabled: 1,
                no_sandbox: 1,
                multi_threaded_message_loop: 1,
                ..Default::default()
            };

            let state = Arc::new(Mutex::new(ProbeState::default()));
            let browser = Arc::new(Mutex::new(None));
            let browser_created = Arc::new(AtomicBool::new(false));
            let browser_create_failed = Arc::new(AtomicBool::new(false));
            let closed = Arc::new(AtomicBool::new(false));

            let handler = ParentBrowserProcessHandler {
                state: state.clone(),
                browser: browser.clone(),
                browser_created: browser_created.clone(),
                browser_create_failed: browser_create_failed.clone(),
                closed: closed.clone(),
                parent_window: sys::HWND(parent_hwnd.0.cast()),
            };
            let mut app = ParentCefAppBuilder::build(handler);
            let initialized = initialize(
                Some(args.as_main_args()),
                Some(&settings),
                Some(&mut app),
                ptr::null_mut(),
            );
            if initialized != 1 {
                return Err(format!("cef_initialize returned {initialized}"));
            }

            let started = Instant::now();
            while !browser_created.load(Ordering::Acquire)
                && !browser_create_failed.load(Ordering::Acquire)
                && started.elapsed() < BROWSER_CREATE_TIMEOUT
            {
                sleep(MESSAGE_PUMP_INTERVAL);
            }
            if browser_create_failed.load(Ordering::Acquire) {
                shutdown();
                return Err("CEF parent-HWND windowless browser creation was rejected".to_string());
            }
            if !browser_created.load(Ordering::Acquire) {
                shutdown();
                return Err("CEF parent-HWND windowless browser creation timed out".to_string());
            }

            println!(
                "CEF OSR Browser created with parent HWND=0x{:X}",
                parent_hwnd.0 as usize
            );

            Ok(Self {
                state,
                browser,
                closed,
            })
        }

        fn request_click(&self, transfer: ClickTransfer) -> bool {
            let Some(browser) = self
                .browser
                .lock()
                .ok()
                .and_then(|slot| slot.as_ref().cloned())
            else {
                return false;
            };
            let mut task = ClickTaskBuilder::build(browser, transfer);
            post_task(ThreadId::UI, Some(&mut task)) == 1
        }

        fn request_composition(&self, transfer: NativeCompositionTransfer) -> bool {
            let Some(browser) = self
                .browser
                .lock()
                .ok()
                .and_then(|slot| slot.as_ref().cloned())
            else {
                return false;
            };
            let mut task = CompositionTaskBuilder::build(browser, transfer);
            post_task(ThreadId::UI, Some(&mut task)) == 1
        }
    }

    impl Drop for ParentProbeRuntime {
        fn drop(&mut self) {
            let browser = self
                .browser
                .lock()
                .ok()
                .and_then(|slot| slot.as_ref().cloned());
            if let Some(browser) = browser {
                let mut task = CloseBrowserTaskBuilder::build(browser);
                if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                    eprintln!("parent-HWND IME probe: failed to post browser close task");
                }
            }
            let started = Instant::now();
            while !self.closed.load(Ordering::Acquire) && started.elapsed() < CLOSE_TIMEOUT {
                sleep(MESSAGE_PUMP_INTERVAL);
            }
            if !self.closed.load(Ordering::Acquire) {
                eprintln!("parent-HWND IME probe: browser close timeout");
            }
            shutdown();
        }
    }

    struct ParentProbeApp {
        runtime: Option<ParentProbeRuntime>,
        init_error: Option<String>,
        texture: Option<egui::TextureHandle>,
        applied_generation: u64,
        browser_active: bool,
        current_click: Option<ClickTransfer>,
        native_ime: Arc<Mutex<NativeImeState>>,
        installed_hwnd: Option<HWND>,
        last_bridge_status: String,
    }

    impl ParentProbeApp {
        fn new() -> Self {
            Self {
                runtime: None,
                init_error: None,
                texture: None,
                applied_generation: 0,
                browser_active: false,
                current_click: None,
                native_ime: Arc::new(Mutex::new(NativeImeState::default())),
                installed_hwnd: None,
                last_bridge_status: "waiting for eframe HWND".to_string(),
            }
        }

        /// eframe window 作成後に HWND を取得し、subclass と CEF Browser を初期化する。
        fn ensure_initialized(&mut self, frame: &eframe::Frame) {
            if self.runtime.is_some() || self.init_error.is_some() {
                return;
            }
            let Some(hwnd) = frame_hwnd(frame) else {
                return;
            };

            unsafe {
                NATIVE_IME_STATE_PTR = Arc::as_ptr(&self.native_ime);
                if SetWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID + 3, 0)
                    .as_bool()
                {
                    self.installed_hwnd = Some(hwnd);
                    println!(
                        "Parent-HWND IME subclass installed: hwnd=0x{:X}",
                        hwnd.0 as usize
                    );
                } else {
                    self.init_error = Some("native IME subclass installation failed".to_string());
                    return;
                }
            }

            match ParentProbeRuntime::new(hwnd) {
                Ok(runtime) => {
                    self.runtime = Some(runtime);
                    self.last_bridge_status = format!(
                        "CEF initialized with parent HWND 0x{:X}",
                        hwnd.0 as usize
                    );
                }
                Err(error) => {
                    self.init_error = Some(error);
                }
            }
        }

        fn update_texture(&mut self, ctx: &egui::Context) {
            let Some(runtime) = self.runtime.as_ref() else {
                return;
            };
            let snapshot = {
                let Ok(state) = runtime.state.lock() else {
                    return;
                };
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
            let Some((generation, width, height, rgba)) = snapshot else {
                return;
            };
            let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
            match self.texture.as_mut() {
                Some(texture) if texture.size() == [width, height] => {
                    texture.set(image, egui::TextureOptions::LINEAR);
                }
                _ => {
                    self.texture = Some(ctx.load_texture(
                        "wv11_04_02_parent_hwnd_ime",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            self.applied_generation = generation;
        }

        fn drain_native_ime_events(&mut self) {
            if !self.browser_active {
                return;
            }
            let Some(runtime) = self.runtime.as_ref() else {
                return;
            };
            let events = self
                .native_ime
                .lock()
                .map(|mut state| state.events.drain(..).collect::<Vec<_>>())
                .unwrap_or_default();
            for event in events {
                match event {
                    NativeImeEvent::Composition(transfer) => {
                        let text = transfer.text.clone();
                        if runtime.request_composition(transfer) {
                            self.last_bridge_status = format!("Composition -> CEF: {text}");
                        }
                    }
                    NativeImeEvent::Result(text) => {
                        println!(
                            "Native IME result observed but not committed in parent-HWND probe: {:?}",
                            text
                        );
                        self.last_bridge_status = format!("Result observed: {text}");
                    }
                    NativeImeEvent::Start => {
                        self.last_bridge_status = "Native composition start".to_string();
                    }
                    NativeImeEvent::End => {
                        self.last_bridge_status = "Native composition end".to_string();
                    }
                }
            }
        }
    }

    impl Drop for ParentProbeApp {
        fn drop(&mut self) {
            // CEF runtime を先に Drop して Browser を閉じ、その後 subclass を解除する。
            self.runtime.take();
            if let Some(hwnd) = self.installed_hwnd.take() {
                unsafe {
                    let _ = RemoveWindowSubclass(
                        hwnd,
                        Some(native_ime_subclass_proc),
                        SUBCLASS_ID + 3,
                    );
                    NATIVE_IME_STATE_PTR = ptr::null();
                }
            }
        }
    }

    impl eframe::App for ParentProbeApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            self.ensure_initialized(frame);
            self.update_texture(ctx);

            if self.browser_active {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
            }

            let (generation, callbacks, bounds) = self
                .runtime
                .as_ref()
                .and_then(|runtime| {
                    runtime.state.lock().ok().map(|state| {
                        (
                            state.generation,
                            state.ime_range_callbacks,
                            state.character_bounds.len(),
                        )
                    })
                })
                .unwrap_or((0, 0, 0));

            let (native_start, native_comp, native_end, compstr, resultstr, cursor) = self
                .native_ime
                .lock()
                .map(|state| {
                    (
                        state.start_count,
                        state.composition_count,
                        state.end_count,
                        state.last_compstr.clone(),
                        state.last_resultstr.clone(),
                        state.last_cursor_pos,
                    )
                })
                .unwrap_or((0, 0, 0, String::new(), String::new(), -1));

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("WV-11-04-02 Parent HWND IME Probe");
                    ui.separator();
                    ui.label(format!("Paint: {generation}"));
                    ui.separator();
                    ui.label(format!("CEF range callbacks: {callbacks}"));
                    ui.separator();
                    ui.label(format!("Bounds: {bounds}"));
                    ui.separator();
                    ui.label(format!(
                        "Native start/comp/end: {native_start}/{native_comp}/{native_end}"
                    ));
                    ui.separator();
                    ui.label(format!("Cursor: {cursor}"));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Native COMPSTR: {:?}", compstr));
                    ui.separator();
                    ui.label(format!("RESULTSTR: {:?}", resultstr));
                    ui.separator();
                    ui.label(format!("Bridge: {}", self.last_bridge_status));
                });
                if let Some(error) = self.init_error.as_ref() {
                    ui.label(format!("Init error: {error}"));
                }
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
                    if response.clicked() {
                        if let Some(pointer) = response.interact_pointer_pos() {
                            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                                self.current_click = Some(ClickTransfer { x, y });
                                self.browser_active = true;
                            }
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Waiting for parent-HWND CEF OSR Paint...")
                    });
                }
            });

            if let (Some(runtime), Some(click)) = (self.runtime.as_ref(), self.current_click) {
                let _ = runtime.request_click(click);
            }
            self.drain_native_ime_events();
            ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
        }
    }

    pub fn run() -> eframe::Result<()> {
        let cli_args: Vec<String> = std::env::args().collect();
        if is_cef_subprocess(&cli_args) {
            std::process::exit(run_subprocess());
        }

        println!("WV-11-04-02 parent HWND CEF OSR IME probe start");
        println!("CEF initialization waits until the eframe HWND exists.");

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 Parent HWND CEF OSR IME Probe")
                .with_inner_size([1100.0, 760.0]),
            ..Default::default()
        };

        eframe::run_native(
            "WV-11-04-02 Parent HWND CEF OSR IME Probe",
            native_options,
            Box::new(move |_cc| Ok(Box::new(ParentProbeApp::new()))),
        )
    }
}

fn main() -> eframe::Result<()> {
    probe::run()
}
