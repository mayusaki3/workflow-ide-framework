mod probe {
    include!("cef_ime_native_bridge_probe.rs");

    use windows::Win32::UI::Input::Ime::{
        ATTR_TARGET_CONVERTED, ATTR_TARGET_NOTCONVERTED, CS_NOMOVECARET, GCS_COMPATTR,
        GCS_COMPCLAUSE,
    };

    #[derive(Clone, Debug)]
    struct DetailedCompositionTransfer {
        text: String,
        cursor_pos: i32,
        attrs: Vec<u8>,
        clauses: Vec<u32>,
        lparam_flags: u32,
    }

    #[derive(Debug, Clone)]
    enum DetailedNativeImeEvent {
        Start,
        Composition(DetailedCompositionTransfer),
        Result(String),
        End,
    }

    #[derive(Debug, Default)]
    struct DetailedNativeImeState {
        start_count: u64,
        composition_count: u64,
        end_count: u64,
        last_compstr: String,
        last_resultstr: String,
        last_cursor_pos: i32,
        last_attrs: Vec<u8>,
        last_clauses: Vec<u32>,
        events: VecDeque<DetailedNativeImeEvent>,
    }

    static mut DETAILED_IME_STATE_PTR: *const Mutex<DetailedNativeImeState> = std::ptr::null();

    /// HIMC から任意のバイト列を読み出す。
    ///
    /// # 引数
    /// - `hwnd`: 対象 HWND。
    /// - `index`: `GCS_COMPATTR` または `GCS_COMPCLAUSE`。
    ///
    /// # 戻り値
    /// - 取得したバイト列。取得できない場合は空配列。
    unsafe fn read_ime_bytes(hwnd: HWND, index: IME_COMPOSITION_STRING) -> Vec<u8> {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return Vec::new();
        }
        let byte_len = ImmGetCompositionStringW(himc, index, None, 0);
        let bytes = if byte_len > 0 {
            let mut buffer = vec![0u8; byte_len as usize];
            let copied = ImmGetCompositionStringW(
                himc,
                index,
                Some(buffer.as_mut_ptr().cast()),
                byte_len as u32,
            );
            if copied > 0 {
                buffer.truncate(copied as usize);
                buffer
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        let _ = ImmReleaseContext(hwnd, himc);
        bytes
    }

    /// `GCS_COMPCLAUSE` の DWORD 配列を取得する。
    unsafe fn read_ime_clauses(hwnd: HWND) -> Vec<u32> {
        read_ime_bytes(hwnd, GCS_COMPCLAUSE)
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    /// cefclient の `IsSelectionAttribute` と同じ条件で target 属性か判定する。
    fn is_target_attr(attr: u8) -> bool {
        attr as u32 == ATTR_TARGET_CONVERTED || attr as u32 == ATTR_TARGET_NOTCONVERTED
    }

    /// Windows native IME の属性・clause・cursor を保持して Bridge queue へ積む。
    unsafe extern "system" fn detailed_ime_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _subclass_id: usize,
        _ref_data: usize,
    ) -> LRESULT {
        if !DETAILED_IME_STATE_PTR.is_null() {
            let state = &*DETAILED_IME_STATE_PTR;
            if let Ok(mut state) = state.lock() {
                match msg {
                    WM_IME_STARTCOMPOSITION => {
                        state.start_count = state.start_count.saturating_add(1);
                        state.events.push_back(DetailedNativeImeEvent::Start);
                        println!("WM_IME_STARTCOMPOSITION count={}", state.start_count);
                    }
                    WM_IME_COMPOSITION => {
                        state.composition_count = state.composition_count.saturating_add(1);
                        let flags = IME_COMPOSITION_STRING(lparam.0 as u32);

                        if flags.contains(GCS_RESULTSTR) {
                            let result = read_ime_string(hwnd, GCS_RESULTSTR);
                            state.last_resultstr = result.clone();
                            if !result.is_empty() {
                                state.events.push_back(DetailedNativeImeEvent::Result(result.clone()));
                            }
                            println!("WM_IME_RESULT result={:?}", result);
                        }

                        if flags.contains(GCS_COMPSTR) {
                            let text = read_ime_string(hwnd, GCS_COMPSTR);
                            let cursor_pos = if flags.contains(GCS_CURSORPOS) {
                                read_cursor_pos(hwnd)
                            } else {
                                0
                            };
                            let attrs = if flags.contains(GCS_COMPATTR) {
                                read_ime_bytes(hwnd, GCS_COMPATTR)
                            } else {
                                Vec::new()
                            };
                            let clauses = if flags.contains(GCS_COMPCLAUSE) {
                                read_ime_clauses(hwnd)
                            } else {
                                Vec::new()
                            };

                            state.last_compstr = text.clone();
                            state.last_cursor_pos = cursor_pos;
                            state.last_attrs = attrs.clone();
                            state.last_clauses = clauses.clone();

                            if !text.is_empty() {
                                state.events.push_back(DetailedNativeImeEvent::Composition(
                                    DetailedCompositionTransfer {
                                        text: text.clone(),
                                        cursor_pos,
                                        attrs: attrs.clone(),
                                        clauses: clauses.clone(),
                                        lparam_flags: lparam.0 as u32,
                                    },
                                ));
                            }
                            println!(
                                "WM_IME_COMPOSITION count={} comp={:?} cursor={} attr={:?} clause={:?} lparam=0x{:X}",
                                state.composition_count,
                                text,
                                cursor_pos,
                                attrs,
                                clauses,
                                lparam.0
                            );
                        }
                    }
                    WM_IME_ENDCOMPOSITION => {
                        state.end_count = state.end_count.saturating_add(1);
                        state.events.push_back(DetailedNativeImeEvent::End);
                        println!("WM_IME_ENDCOMPOSITION count={}", state.end_count);
                    }
                    _ => {}
                }
            }
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    /// cefclient `GetCompositionInfo` と同じ規則で underline と selection range を生成する。
    ///
    /// target 属性は太線、通常 clause は細線にする。selection range は Windows IMM32
    /// の cursor 値を composition_start として `start..start+text_len` を CEF へ渡す。
    ///
    /// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
    fn build_cefclient_style_composition(
        transfer: &DetailedCompositionTransfer,
    ) -> (Vec<CompositionUnderline>, Range, u32, u32) {
        let text_len = transfer.text.encode_utf16().count() as u32;

        let mut target_start = text_len;
        let mut target_end = text_len;
        if !transfer.attrs.is_empty() {
            let mut start = 0usize;
            while start < transfer.attrs.len() && !is_target_attr(transfer.attrs[start]) {
                start += 1;
            }
            let mut end = start;
            while end < transfer.attrs.len() && is_target_attr(transfer.attrs[end]) {
                end += 1;
            }
            target_start = start as u32;
            target_end = end as u32;
        }

        let mut underlines = Vec::new();
        if transfer.clauses.len() >= 2 {
            for clause in transfer.clauses.windows(2) {
                let from = clause[0].min(text_len);
                let to = clause[1].min(text_len);
                if to <= from {
                    continue;
                }
                let thick = from >= target_start && to <= target_end;
                underlines.push(CompositionUnderline {
                    range: Range { from, to },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: thick.into(),
                    ..Default::default()
                });
            }
        }

        if underlines.is_empty() {
            if target_start > 0 {
                underlines.push(CompositionUnderline {
                    range: Range { from: 0, to: target_start.min(text_len) },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: 0,
                    ..Default::default()
                });
            }
            if target_end > target_start {
                underlines.push(CompositionUnderline {
                    range: Range {
                        from: target_start.min(text_len),
                        to: target_end.min(text_len),
                    },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: 1,
                    ..Default::default()
                });
            }
            if target_end < text_len {
                underlines.push(CompositionUnderline {
                    range: Range { from: target_end, to: text_len },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: 0,
                    ..Default::default()
                });
            }
            if underlines.is_empty() && text_len > 0 {
                underlines.push(CompositionUnderline {
                    range: Range { from: 0, to: text_len },
                    color: 0xFF000000,
                    background_color: 0x00000000,
                    thick: 0,
                    ..Default::default()
                });
            }
        }

        let has_no_move_caret = transfer.lparam_flags & (CS_NOMOVECARET as u32) != 0;
        let has_cursor = IME_COMPOSITION_STRING(transfer.lparam_flags).contains(GCS_CURSORPOS);
        let composition_start = if !has_no_move_caret && has_cursor {
            transfer.cursor_pos.max(0) as u32
        } else {
            0
        };
        let selection = Range {
            from: composition_start,
            to: composition_start.saturating_add(text_len),
        };

        (underlines, selection, target_start, target_end)
    }

    /// 詳細 native IME state を CEF OSR Composition へ転送する。
    fn send_detailed_composition(browser: &Browser, transfer: &DetailedCompositionTransfer) {
        if transfer.text.is_empty() {
            return;
        }
        let Some(host) = browser.host() else { return; };
        let (underlines, selection, target_start, target_end) =
            build_cefclient_style_composition(transfer);
        let text = CefString::from(transfer.text.as_str());

        host.ime_set_composition(
            Some(&text),
            Some(&underlines),
            None,
            Some(&selection),
        );

        println!(
            "CEF cefclient-style composition set: {:?} cursor={} target={}..{} selection={}..{} underlines={:?}",
            transfer.text,
            transfer.cursor_pos,
            target_start,
            target_end,
            selection.from,
            selection.to,
            underlines
                .iter()
                .map(|u| (u.range.from, u.range.to, u.thick))
                .collect::<Vec<_>>()
        );
    }

    #[derive(Clone)]
    struct DetailedCompositionTask {
        browser: Browser,
        transfer: DetailedCompositionTransfer,
    }

    wrap_task! {
        struct DetailedCompositionTaskBuilder { task: DetailedCompositionTask, }
        impl Task {
            fn execute(&self) {
                send_detailed_composition(&self.task.browser, &self.task.transfer);
            }
        }
    }

    impl DetailedCompositionTaskBuilder {
        fn build(browser: Browser, transfer: DetailedCompositionTransfer) -> Task {
            Self::new(DetailedCompositionTask { browser, transfer })
        }
    }

    fn request_detailed_composition(
        runtime: &ProbeRuntime,
        transfer: DetailedCompositionTransfer,
    ) -> bool {
        let Some(browser) = runtime
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned())
        else {
            return false;
        };
        let mut task = DetailedCompositionTaskBuilder::build(browser, transfer);
        post_task(ThreadId::UI, Some(&mut task)) == 1
    }

    struct DetailedProbeApp {
        runtime: ProbeRuntime,
        texture: Option<egui::TextureHandle>,
        applied_generation: u64,
        browser_active: bool,
        current_click: Option<ClickTransfer>,
        native_ime: Arc<Mutex<DetailedNativeImeState>>,
        installed_hwnd: Option<HWND>,
        last_bridge_status: String,
    }

    impl DetailedProbeApp {
        fn new(runtime: ProbeRuntime) -> Self {
            Self {
                runtime,
                texture: None,
                applied_generation: 0,
                browser_active: false,
                current_click: None,
                native_ime: Arc::new(Mutex::new(DetailedNativeImeState::default())),
                installed_hwnd: None,
                last_bridge_status: "waiting".to_string(),
            }
        }

        fn ensure_subclass(&mut self, frame: &eframe::Frame) {
            if self.installed_hwnd.is_some() {
                return;
            }
            let Some(hwnd) = frame_hwnd(frame) else { return; };
            unsafe {
                DETAILED_IME_STATE_PTR = Arc::as_ptr(&self.native_ime);
                if SetWindowSubclass(hwnd, Some(detailed_ime_subclass_proc), SUBCLASS_ID + 2, 0)
                    .as_bool()
                {
                    self.installed_hwnd = Some(hwnd);
                    println!("Detailed IME bridge subclass installed: hwnd=0x{:X}", hwnd.0 as usize);
                }
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
                        "wv11_04_02_native_detail_bridge",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            self.applied_generation = generation;
        }

        fn drain_events(&mut self) {
            if !self.browser_active {
                return;
            }
            let events = self
                .native_ime
                .lock()
                .map(|mut state| state.events.drain(..).collect::<Vec<_>>())
                .unwrap_or_default();
            for event in events {
                match event {
                    DetailedNativeImeEvent::Composition(transfer) => {
                        let text = transfer.text.clone();
                        if request_detailed_composition(&self.runtime, transfer) {
                            self.last_bridge_status = format!("Detailed Composition -> CEF: {text}");
                        }
                    }
                    DetailedNativeImeEvent::Result(text) => {
                        println!("Native IME result observed but not committed: {:?}", text);
                        self.last_bridge_status = format!("Result observed: {text}");
                    }
                    DetailedNativeImeEvent::Start => {
                        self.last_bridge_status = "Native composition start".to_string();
                    }
                    DetailedNativeImeEvent::End => {
                        self.last_bridge_status = "Native composition end".to_string();
                    }
                }
            }
        }
    }

    impl Drop for DetailedProbeApp {
        fn drop(&mut self) {
            if let Some(hwnd) = self.installed_hwnd.take() {
                unsafe {
                    let _ = RemoveWindowSubclass(
                        hwnd,
                        Some(detailed_ime_subclass_proc),
                        SUBCLASS_ID + 2,
                    );
                    DETAILED_IME_STATE_PTR = ptr::null();
                }
            }
        }
    }

    impl eframe::App for DetailedProbeApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            self.ensure_subclass(frame);
            self.update_texture(ctx);

            if self.browser_active {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
            }

            let (generation, callbacks, bounds) = self
                .runtime
                .state
                .lock()
                .map(|state| {
                    (
                        state.generation,
                        state.ime_range_callbacks,
                        state.character_bounds.len(),
                    )
                })
                .unwrap_or((0, 0, 0));

            let (start, comp, end, text, cursor, attrs, clauses) = self
                .native_ime
                .lock()
                .map(|state| {
                    (
                        state.start_count,
                        state.composition_count,
                        state.end_count,
                        state.last_compstr.clone(),
                        state.last_cursor_pos,
                        state.last_attrs.clone(),
                        state.last_clauses.clone(),
                    )
                })
                .unwrap_or((0, 0, 0, String::new(), -1, Vec::new(), Vec::new()));

            egui::TopBottomPanel::top("detail_status").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("WV-11-04-02 cefclient-style IME Detail Bridge");
                    ui.separator();
                    ui.label(format!("Paint: {generation}"));
                    ui.separator();
                    ui.label(format!("CEF range callbacks: {callbacks}"));
                    ui.separator();
                    ui.label(format!("Bounds: {bounds}"));
                    ui.separator();
                    ui.label(format!("Native start/comp/end: {start}/{comp}/{end}"));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("COMPSTR: {:?}", text));
                    ui.separator();
                    ui.label(format!("Cursor: {cursor}"));
                    ui.separator();
                    ui.label(format!("Attr: {:?}", attrs));
                    ui.separator();
                    ui.label(format!("Clause: {:?}", clauses));
                });
                ui.label(format!("Bridge: {}", self.last_bridge_status));
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
                    ui.centered_and_justified(|ui| ui.label("Waiting for CEF OSR Paint..."));
                }
            });

            if let Some(click) = self.current_click {
                let _ = self.runtime.request_click(click);
            }
            self.drain_events();
            ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
        }
    }

    pub fn run() -> eframe::Result<()> {
        let cli_args: Vec<String> = std::env::args().collect();
        if is_cef_subprocess(&cli_args) {
            std::process::exit(run_subprocess());
        }

        println!("WV-11-04-02 cefclient-style native IME detail bridge probe start");
        println!("Click Browser input, type Japanese text, then press Space repeatedly.");

        let runtime = match ProbeRuntime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("detail bridge probe failed: {error}");
                std::process::exit(1);
            }
        };

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 cefclient-style IME Detail Bridge Probe")
                .with_inner_size([1100.0, 760.0]),
            ..Default::default()
        };

        eframe::run_native(
            "WV-11-04-02 cefclient-style IME Detail Bridge Probe",
            native_options,
            Box::new(move |_cc| Ok(Box::new(DetailedProbeApp::new(runtime)))),
        )
    }
}

fn main() -> eframe::Result<()> {
    probe::run()
}