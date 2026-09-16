// WV-11-04-02 CEF UI-thread child HWND IME Probe。
//
// 役割:
// - CEF UI thread 上に native child HWND を作成し、Windows IME と CEF OSR を直接接続する。
// - WM_IME_* 処理中に CEF ImeSetComposition / ImeCommitText を呼び出す。
// - eframe の close request を一旦保留し、CEF Browser の OnBeforeClose 完了後に window を閉じる。
//
// 注意点:
// - 本ファイルは技術検証用であり、正式 Surface API ではない。
// - Candidate Window 位置同期は後続検証とする。
// - 終了順序は close request -> CEF close -> OnBeforeClose 内 child HWND 破棄 -> eframe close -> CefShutdown。

mod probe {
    include!("cef_ime_native_bridge_probe.rs");

    use std::ffi::c_void;
    use std::sync::atomic::AtomicUsize;
    use windows::core::w;
    use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, WS_CHILD, WS_VISIBLE, WINDOW_EX_STYLE,
    };

    const CHILD_SUBCLASS_ID: usize = SUBCLASS_ID + 10;

    #[derive(Default)]
    struct UiChildImeState {
        browser: Arc<Mutex<Option<Browser>>>,
        start_count: u64,
        composition_count: u64,
        end_count: u64,
        last_compstr: String,
        last_resultstr: String,
        last_cursor_pos: i32,
    }

    static mut UI_CHILD_IME_STATE_PTR: *const Mutex<UiChildImeState> = ptr::null();

    unsafe extern "system" fn ui_child_ime_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _subclass_id: usize,
        _ref_data: usize,
    ) -> LRESULT {
        if UI_CHILD_IME_STATE_PTR.is_null() {
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }

        let state_mutex = &*UI_CHILD_IME_STATE_PTR;
        match msg {
            WM_IME_SETCONTEXT => return DefSubclassProc(hwnd, msg, wparam, lparam),
            WM_IME_STARTCOMPOSITION => {
                if let Ok(mut state) = state_mutex.lock() {
                    state.start_count = state.start_count.saturating_add(1);
                    println!(
                        "UI-THREAD CHILD WM_IME_STARTCOMPOSITION count={} consumed=true",
                        state.start_count
                    );
                }
                return LRESULT(0);
            }
            WM_IME_COMPOSITION => {
                let flags = IME_COMPOSITION_STRING(lparam.0 as u32);
                let result = if flags.contains(GCS_RESULTSTR) {
                    read_ime_string(hwnd, GCS_RESULTSTR)
                } else {
                    String::new()
                };
                let comp = if flags.contains(GCS_COMPSTR) {
                    read_ime_string(hwnd, GCS_COMPSTR)
                } else {
                    String::new()
                };
                let cursor = if flags.contains(GCS_CURSORPOS) {
                    read_cursor_pos(hwnd)
                } else {
                    0
                };

                let browser = {
                    let Ok(mut state) = state_mutex.lock() else {
                        return LRESULT(0);
                    };
                    state.composition_count = state.composition_count.saturating_add(1);
                    if !result.is_empty() {
                        state.last_resultstr = result.clone();
                    }
                    if !comp.is_empty() {
                        state.last_compstr = comp.clone();
                        state.last_cursor_pos = cursor;
                    }
                    println!(
                        "UI-THREAD CHILD WM_IME_COMPOSITION count={} comp={:?} result={:?} cursor={} lparam=0x{:X} consumed=true",
                        state.composition_count, comp, result, cursor, lparam.0
                    );
                    state.browser.lock().ok().and_then(|slot| slot.as_ref().cloned())
                };

                if let Some(browser) = browser {
                    if let Some(host) = browser.host() {
                        if !result.is_empty() {
                            let text = CefString::from(result.as_str());
                            host.ime_commit_text(Some(&text), None, 0);
                            println!("UI-THREAD CHILD CEF IME commit: {:?}", result);
                        }
                        if !comp.is_empty() {
                            let text_len = comp.encode_utf16().count() as u32;
                            let range = Range { from: 0, to: text_len };
                            let underlines = [CompositionUnderline {
                                range: range.clone(),
                                color: 0xFF000000,
                                background_color: 0x00000000,
                                thick: 0,
                                ..Default::default()
                            }];
                            let text = CefString::from(comp.as_str());
                            host.ime_set_composition(
                                Some(&text),
                                Some(&underlines),
                                Some(&range),
                                Some(&range),
                            );
                            println!(
                                "UI-THREAD CHILD CEF IME composition direct: {:?} range=0..{}",
                                comp, text_len
                            );
                        }
                    }
                }
                return LRESULT(0);
            }
            WM_IME_ENDCOMPOSITION => {
                if let Ok(mut state) = state_mutex.lock() {
                    state.end_count = state.end_count.saturating_add(1);
                    println!(
                        "UI-THREAD CHILD WM_IME_ENDCOMPOSITION count={} consumed=true",
                        state.end_count
                    );
                }
                return LRESULT(0);
            }
            _ => {}
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    fn create_ui_child_window(parent: sys::HWND) -> Result<HWND, String> {
        let parent = HWND(parent.0.cast());
        let child = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!(""),
                WS_CHILD | WS_VISIBLE,
                0,
                0,
                1,
                1,
                Some(parent),
                None,
                None,
                None,
            )
        }
        .map_err(|error| format!("CreateWindowExW for CEF IME child failed: {error}"))?;

        let installed = unsafe {
            SetWindowSubclass(
                child,
                Some(ui_child_ime_subclass_proc),
                CHILD_SUBCLASS_ID,
                0,
            )
        };
        if !installed.as_bool() {
            unsafe { let _ = DestroyWindow(child); }
            return Err("SetWindowSubclass for CEF IME child failed".to_string());
        }
        println!(
            "CEF UI-thread IME child HWND created: parent=0x{:X} child=0x{:X}",
            parent.0 as usize,
            child.0 as usize
        );
        Ok(child)
    }

    #[derive(Clone)]
    struct UiChildLifeSpanHandler {
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
        child_hwnd: Arc<AtomicUsize>,
    }

    wrap_life_span_handler! {
        struct UiChildLifeSpanHandlerBuilder { handler: UiChildLifeSpanHandler, }
        impl LifeSpanHandler {
            fn on_after_created(&self, browser: Option<&mut Browser>) {
                let Some(browser) = browser.cloned() else { return; };
                if let Ok(mut slot) = self.handler.browser.lock() {
                    *slot = Some(browser);
                    self.handler.browser_created.store(true, Ordering::Release);
                }
            }

            fn on_before_close(&self, _browser: Option<&mut Browser>) {
                if let Ok(mut slot) = self.handler.browser.lock() {
                    *slot = None;
                }
                let child = self.handler.child_hwnd.swap(0, Ordering::AcqRel);
                if child != 0 {
                    let hwnd = HWND(child as *mut c_void);
                    unsafe {
                        let _ = RemoveWindowSubclass(
                            hwnd,
                            Some(ui_child_ime_subclass_proc),
                            CHILD_SUBCLASS_ID,
                        );
                        let _ = DestroyWindow(hwnd);
                    }
                    println!("CEF UI-thread IME child HWND destroyed in OnBeforeClose");
                }
                self.handler.closed.store(true, Ordering::Release);
                println!("CEF OnBeforeClose completed with child destruction");
            }
        }
    }

    impl UiChildLifeSpanHandlerBuilder {
        fn build(
            browser: Arc<Mutex<Option<Browser>>>,
            browser_created: Arc<AtomicBool>,
            closed: Arc<AtomicBool>,
            child_hwnd: Arc<AtomicUsize>,
        ) -> LifeSpanHandler {
            Self::new(UiChildLifeSpanHandler {
                browser,
                browser_created,
                closed,
                child_hwnd,
            })
        }
    }

    wrap_client! {
        struct UiChildClientBuilder {
            render_handler: RenderHandler,
            life_span_handler: LifeSpanHandler,
        }
        impl Client {
            fn render_handler(&self) -> Option<RenderHandler> {
                Some(self.render_handler.clone())
            }
            fn life_span_handler(&self) -> Option<LifeSpanHandler> {
                Some(self.life_span_handler.clone())
            }
        }
    }

    impl UiChildClientBuilder {
        fn build(
            state: Arc<Mutex<ProbeState>>,
            browser: Arc<Mutex<Option<Browser>>>,
            browser_created: Arc<AtomicBool>,
            closed: Arc<AtomicBool>,
            child_hwnd: Arc<AtomicUsize>,
        ) -> Client {
            Self::new(
                ProbeRenderHandlerBuilder::build(state),
                UiChildLifeSpanHandlerBuilder::build(
                    browser,
                    browser_created,
                    closed,
                    child_hwnd,
                ),
            )
        }
    }

    #[derive(Clone)]
    struct UiChildBrowserProcessHandler {
        state: Arc<Mutex<ProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        browser_create_failed: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
        parent_window: sys::HWND,
        child_hwnd: Arc<AtomicUsize>,
    }

    wrap_browser_process_handler! {
        struct UiChildBrowserProcessHandlerBuilder { handler: UiChildBrowserProcessHandler, }
        impl BrowserProcessHandler {
            fn on_context_initialized(&self) {
                let child = match create_ui_child_window(self.handler.parent_window) {
                    Ok(child) => child,
                    Err(error) => {
                        eprintln!("{error}");
                        self.handler.browser_create_failed.store(true, Ordering::Release);
                        return;
                    }
                };
                self.handler.child_hwnd.store(child.0 as usize, Ordering::Release);
                let mut client = UiChildClientBuilder::build(
                    self.handler.state.clone(),
                    self.handler.browser.clone(),
                    self.handler.browser_created.clone(),
                    self.handler.closed.clone(),
                    self.handler.child_hwnd.clone(),
                );
                let window_info = WindowInfo::default().set_as_windowless(sys::HWND(child.0.cast()));
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
                    self.handler.browser_create_failed.store(true, Ordering::Release);
                }
            }
        }
    }

    impl UiChildBrowserProcessHandlerBuilder {
        fn build(handler: UiChildBrowserProcessHandler) -> BrowserProcessHandler {
            Self::new(handler)
        }
    }

    #[derive(Clone)]
    struct UiChildCefApp { browser_process_handler: BrowserProcessHandler, }

    wrap_app! {
        struct UiChildCefAppBuilder { handler: UiChildCefApp, }
        impl App {
            fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
                Some(self.handler.browser_process_handler.clone())
            }
        }
    }

    impl UiChildCefAppBuilder {
        fn build(handler: UiChildBrowserProcessHandler) -> App {
            Self::new(UiChildCefApp {
                browser_process_handler: UiChildBrowserProcessHandlerBuilder::build(handler),
            })
        }
    }

    fn send_focus_click(browser: &Browser, transfer: ClickTransfer) {
        let Some(host) = browser.host() else { return; };
        host.set_focus(true.into());
        let event = MouseEvent { x: transfer.x, y: transfer.y, ..Default::default() };
        host.send_mouse_move_event(Some(&event), 0);
        host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 0, 1);
        host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 1, 1);
        println!("CEF UI-thread Browser focus click sent: x={} y={}", transfer.x, transfer.y);
    }

    #[derive(Clone)]
    struct UiChildClickTask { browser: Browser, child_hwnd: usize, transfer: ClickTransfer, }

    wrap_task! {
        struct UiChildClickTaskBuilder { task: UiChildClickTask, }
        impl Task {
            fn execute(&self) {
                let hwnd = HWND(self.task.child_hwnd as *mut c_void);
                unsafe {
                    match SetFocus(Some(hwnd)) {
                        Ok(previous) => println!(
                            "CEF UI-thread child native focus set: child=0x{:X} previous=0x{:X}",
                            hwnd.0 as usize, previous.0 as usize
                        ),
                        Err(error) => eprintln!("SetFocus(child) failed: {error}"),
                    }
                }
                send_focus_click(&self.task.browser, self.task.transfer);
            }
        }
    }

    impl UiChildClickTaskBuilder {
        fn build(browser: Browser, child_hwnd: usize, transfer: ClickTransfer) -> Task {
            Self::new(UiChildClickTask { browser, child_hwnd, transfer })
        }
    }

    #[derive(Clone)]
    struct PrepareCloseTask { browser: Browser, }

    wrap_task! {
        struct PrepareCloseTaskBuilder { task: PrepareCloseTask, }
        impl Task {
            fn execute(&self) {
                if let Some(host) = self.task.browser.host() {
                    host.ime_cancel_composition();
                    host.set_focus(false.into());
                    unsafe { let _ = SetFocus(None); }
                    println!("CEF UI-thread IME canceled and focus cleared");
                    host.close_browser(true.into());
                    println!("CEF UI-thread browser close requested before eframe teardown");
                }
            }
        }
    }

    impl PrepareCloseTaskBuilder {
        fn build(browser: Browser) -> Task { Self::new(PrepareCloseTask { browser }) }
    }

    struct UiChildRuntime {
        state: Arc<Mutex<ProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        browser_create_failed: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
        close_started: AtomicBool,
        child_hwnd: Arc<AtomicUsize>,
        ime_state: Arc<Mutex<UiChildImeState>>,
    }

    impl UiChildRuntime {
        fn new(parent_hwnd: HWND) -> Result<Self, String> {
            let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
            let args = args::Args::new();
            let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
            if process_result >= 0 {
                return Err(format!("Unexpected subprocess result: {process_result}"));
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
            let child_hwnd = Arc::new(AtomicUsize::new(0));
            let ime_state = Arc::new(Mutex::new(UiChildImeState {
                browser: browser.clone(),
                ..Default::default()
            }));
            unsafe { UI_CHILD_IME_STATE_PTR = Arc::as_ptr(&ime_state); }

            let handler = UiChildBrowserProcessHandler {
                state: state.clone(),
                browser: browser.clone(),
                browser_created: browser_created.clone(),
                browser_create_failed: browser_create_failed.clone(),
                closed: closed.clone(),
                parent_window: sys::HWND(parent_hwnd.0.cast()),
                child_hwnd: child_hwnd.clone(),
            };
            let mut app = UiChildCefAppBuilder::build(handler);
            let initialized = initialize(
                Some(args.as_main_args()),
                Some(&settings),
                Some(&mut app),
                ptr::null_mut(),
            );
            if initialized != 1 {
                unsafe { UI_CHILD_IME_STATE_PTR = ptr::null(); }
                return Err(format!("cef_initialize returned {initialized}"));
            }
            println!("CEF UI-thread child browser creation started asynchronously");
            Ok(Self {
                state,
                browser,
                browser_created,
                browser_create_failed,
                closed,
                close_started: AtomicBool::new(false),
                child_hwnd,
                ime_state,
            })
        }

        fn request_click(&self, transfer: ClickTransfer) -> bool {
            let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
                return false;
            };
            let child = self.child_hwnd.load(Ordering::Acquire);
            if child == 0 { return false; }
            let mut task = UiChildClickTaskBuilder::build(browser, child, transfer);
            post_task(ThreadId::UI, Some(&mut task)) == 1
        }

        /// eframe の event loop が生きている update 中に CEF close を開始する。
        fn request_close(&self) -> bool {
            if self.close_started.swap(true, Ordering::AcqRel) {
                return true;
            }
            let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
                self.closed.store(true, Ordering::Release);
                return true;
            };
            let mut task = PrepareCloseTaskBuilder::build(browser);
            if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                self.close_started.store(false, Ordering::Release);
                eprintln!("Failed to post CEF UI-thread prepare-close task");
                return false;
            }
            println!("CEF close sequence started while eframe event loop is active");
            true
        }
    }

    impl Drop for UiChildRuntime {
        fn drop(&mut self) {
            // DropではCEF UI threadへのpost/waitを行わない。OnBeforeCloseはupdate中に完了済みであること。
            if !self.closed.load(Ordering::Acquire) {
                eprintln!("CEF Browser not closed before runtime drop; skip CefShutdown");
                unsafe { UI_CHILD_IME_STATE_PTR = ptr::null(); }
                return;
            }
            unsafe { UI_CHILD_IME_STATE_PTR = ptr::null(); }
            println!("CEF shutdown start after eframe-approved browser close");
            shutdown();
            println!("CEF shutdown complete");
        }
    }

    struct UiChildProbeApp {
        runtime: Option<UiChildRuntime>,
        init_error: Option<String>,
        texture: Option<egui::TextureHandle>,
        applied_generation: u64,
        current_click: Option<ClickTransfer>,
        init_started_at: Option<Instant>,
        close_requested_by_user: bool,
    }

    impl UiChildProbeApp {
        fn new() -> Self {
            Self {
                runtime: None,
                init_error: None,
                texture: None,
                applied_generation: 0,
                current_click: None,
                init_started_at: None,
                close_requested_by_user: false,
            }
        }

        fn ensure_initialized(&mut self, frame: &eframe::Frame) {
            if self.runtime.is_some() || self.init_error.is_some() { return; }
            let Some(hwnd) = frame_hwnd(frame) else { return; };
            match UiChildRuntime::new(hwnd) {
                Ok(runtime) => {
                    self.runtime = Some(runtime);
                    self.init_started_at = Some(Instant::now());
                }
                Err(error) => self.init_error = Some(error),
            }
        }

        fn check_initialization(&mut self) {
            let Some(runtime) = self.runtime.as_ref() else { return; };
            if runtime.browser_create_failed.load(Ordering::Acquire) {
                self.init_error = Some("CEF UI-thread child browser creation was rejected".to_string());
            } else if !runtime.browser_created.load(Ordering::Acquire)
                && self.init_started_at.map(|t| t.elapsed() >= BROWSER_CREATE_TIMEOUT).unwrap_or(false)
            {
                self.init_error = Some("CEF UI-thread child browser creation timed out".to_string());
            }
        }

        fn update_texture(&mut self, ctx: &egui::Context) {
            let Some(runtime) = self.runtime.as_ref() else { return; };
            let snapshot = {
                let Ok(state) = runtime.state.lock() else { return; };
                if state.generation == self.applied_generation || state.width <= 0 || state.height <= 0 || state.rgba.is_empty() {
                    None
                } else {
                    Some((state.generation, state.width as usize, state.height as usize, state.rgba.clone()))
                }
            };
            let Some((generation, width, height, rgba)) = snapshot else { return; };
            let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
            match self.texture.as_mut() {
                Some(texture) if texture.size() == [width, height] => texture.set(image, egui::TextureOptions::LINEAR),
                _ => self.texture = Some(ctx.load_texture("wv11_04_02_ui_child_ime", image, egui::TextureOptions::LINEAR)),
            }
            self.applied_generation = generation;
        }

        fn handle_close(&mut self, ctx: &egui::Context) {
            let native_close = ctx.input(|input| input.viewport().close_requested());
            if native_close {
                let cef_closed = self.runtime.as_ref().map(|r| r.closed.load(Ordering::Acquire)).unwrap_or(true);
                if !cef_closed {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    if !self.close_requested_by_user {
                        self.close_requested_by_user = true;
                        if let Some(runtime) = self.runtime.as_ref() {
                            let _ = runtime.request_close();
                        }
                    }
                }
            }

            if self.close_requested_by_user {
                let cef_closed = self.runtime.as_ref().map(|r| r.closed.load(Ordering::Acquire)).unwrap_or(true);
                if cef_closed {
                    println!("CEF close completed; allow eframe root viewport to close");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    impl eframe::App for UiChildProbeApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            self.ensure_initialized(frame);
            self.check_initialization();
            self.handle_close(ctx);
            self.update_texture(ctx);

            let (generation, callbacks, bounds, start, comp, end, compstr, resultstr, cursor, created, closing) = self.runtime.as_ref().map(|runtime| {
                let (generation, callbacks, bounds) = runtime.state.lock().map(|state| (
                    state.generation, state.ime_range_callbacks, state.character_bounds.len()
                )).unwrap_or((0, 0, 0));
                let (start, comp, end, compstr, resultstr, cursor) = runtime.ime_state.lock().map(|state| (
                    state.start_count, state.composition_count, state.end_count,
                    state.last_compstr.clone(), state.last_resultstr.clone(), state.last_cursor_pos
                )).unwrap_or((0, 0, 0, String::new(), String::new(), -1));
                (
                    generation, callbacks, bounds, start, comp, end, compstr, resultstr, cursor,
                    runtime.browser_created.load(Ordering::Acquire), runtime.close_started.load(Ordering::Acquire)
                )
            }).unwrap_or((0, 0, 0, 0, 0, 0, String::new(), String::new(), -1, false, false));

            egui::TopBottomPanel::top("status").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("WV-11-04-02 CEF UI-thread child HWND IME Probe");
                    ui.separator(); ui.label(format!("Browser created: {created}"));
                    ui.separator(); ui.label(format!("Paint: {generation}"));
                    ui.separator(); ui.label(format!("CEF range callbacks: {callbacks}"));
                    ui.separator(); ui.label(format!("Bounds: {bounds}"));
                    ui.separator(); ui.label(format!("Native start/comp/end: {start}/{comp}/{end}"));
                    ui.separator(); ui.label(format!("Cursor: {cursor}"));
                    if closing { ui.separator(); ui.label("Closing CEF..."); }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("COMPSTR: {:?}", compstr));
                    ui.separator(); ui.label(format!("RESULTSTR: {:?}", resultstr));
                });
                if let Some(error) = self.init_error.as_ref() {
                    ui.colored_label(egui::Color32::RED, format!("Init error: {error}"));
                }
            });

            self.current_click = None;
            egui::CentralPanel::default().show(ctx, |ui| {
                let available = ui.available_size();
                if let Some(texture) = self.texture.as_ref() {
                    let response = ui.add(
                        egui::Image::new(texture).fit_to_exact_size(available).sense(egui::Sense::click())
                    );
                    if response.clicked() {
                        if let Some(pointer) = response.interact_pointer_pos() {
                            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                                self.current_click = Some(ClickTransfer { x, y });
                            }
                        }
                    }
                } else if created {
                    ui.centered_and_justified(|ui| ui.label("Waiting for CEF OSR Paint..."));
                } else {
                    ui.centered_and_justified(|ui| ui.label("Starting CEF UI-thread child browser..."));
                }
            });

            if !closing {
                if let (Some(runtime), Some(click)) = (self.runtime.as_ref(), self.current_click) {
                    let _ = runtime.request_click(click);
                }
            }
            ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
        }
    }

    pub fn run() -> eframe::Result<()> {
        let cli_args: Vec<String> = std::env::args().collect();
        if is_cef_subprocess(&cli_args) {
            std::process::exit(run_subprocess());
        }
        println!("WV-11-04-02 CEF UI-thread child HWND IME probe start");
        println!("Click Browser input, enable Japanese IME, and type without confirming.");
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 CEF UI-thread child HWND IME Probe")
                .with_inner_size([1100.0, 760.0]),
            ..Default::default()
        };
        eframe::run_native(
            "WV-11-04-02 CEF UI-thread child HWND IME Probe",
            native_options,
            Box::new(move |_cc| Ok(Box::new(UiChildProbeApp::new()))),
        )
    }
}

fn main() -> eframe::Result<()> {
    probe::run()
}
