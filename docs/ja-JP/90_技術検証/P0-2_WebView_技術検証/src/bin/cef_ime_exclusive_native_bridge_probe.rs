// WV-11-04-02 Exclusive Windows native IME -> CEF bridge Probe。
//
// 役割:
// - 既存の native IME -> CEF bridge Probe を再利用する。
// - Browser Surface が active な間、WM_IME_STARTCOMPOSITION / WM_IME_COMPOSITION /
//   WM_IME_ENDCOMPOSITION を DefSubclassProc より前に排他的に消費する。
// - native IME と winit/egui の二重 Composition 処理を除外した状態で、
//   CEF OSR の compositionstart / compositionupdate が成立するかを切り分ける。
//
// 注意点:
// - WM_IME_SETCONTEXT は native IME の状態維持のため通常処理へ流す。
// - GCS_RESULTSTR は本 Probe では記録のみで、CEF へ Commit しない。
// - Candidate Window の位置同期は本 Probe の判定対象外。
// - 正式 Surface API ではない。

mod probe {
    include!("cef_ime_native_bridge_probe.rs");

    const EXCLUSIVE_SUBCLASS_ID: usize = 0x5749_4D47;

    /// Browser active 中の Composition 系 Windows message を、既存 bridge subclass より先に
    /// 観測して queue へ積み、その場で消費する。
    ///
    /// WM_IME_SETCONTEXT と Browser inactive 中の message は通常処理へ流す。
    unsafe extern "system" fn exclusive_native_ime_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _subclass_id: usize,
        _ref_data: usize,
    ) -> LRESULT {
        if !NATIVE_IME_STATE_PTR.is_null() {
            let shared = &*NATIVE_IME_STATE_PTR;
            if let Ok(mut state) = shared.lock() {
                // browser_active は既存 ProbeEframeApp 側で管理されるため、
                // HWND が登録済みで IME queue の利用が開始されていることを active 条件とする。
                // 実際の subclass は Browser click 後にのみ追加するため、この時点では常に active。
                match msg {
                    WM_IME_STARTCOMPOSITION => {
                        state.start_count = state.start_count.saturating_add(1);
                        state.ime_open = read_ime_open(hwnd);
                        state.events.push_back(NativeImeEvent::Start);
                        println!(
                            "EXCLUSIVE WM_IME_STARTCOMPOSITION count={} open={} consumed=true",
                            state.start_count, state.ime_open
                        );
                        return LRESULT(0);
                    }
                    WM_IME_COMPOSITION => {
                        state.composition_count = state.composition_count.saturating_add(1);
                        state.ime_open = read_ime_open(hwnd);
                        let flags = IME_COMPOSITION_STRING(lparam.0 as u32);
                        if flags.contains(GCS_COMPSTR) {
                            let text = read_ime_string(hwnd, GCS_COMPSTR);
                            let cursor_pos = read_cursor_pos(hwnd);
                            state.last_compstr = text.clone();
                            state.last_cursor_pos = cursor_pos;
                            if !text.is_empty() {
                                state.events.push_back(NativeImeEvent::Composition(
                                    NativeCompositionTransfer {
                                        text: text.clone(),
                                        cursor_pos,
                                    },
                                ));
                            }
                            println!(
                                "EXCLUSIVE WM_IME_COMPOSITION count={} comp={:?} cursor={} lparam=0x{:X} consumed=true",
                                state.composition_count, text, cursor_pos, lparam.0
                            );
                        }
                        if flags.contains(GCS_RESULTSTR) {
                            let result = read_ime_string(hwnd, GCS_RESULTSTR);
                            state.last_resultstr = result.clone();
                            if !result.is_empty() {
                                state.events.push_back(NativeImeEvent::Result(result.clone()));
                            }
                            println!("EXCLUSIVE WM_IME_RESULT result={:?} consumed=true", result);
                        }
                        return LRESULT(0);
                    }
                    WM_IME_ENDCOMPOSITION => {
                        state.end_count = state.end_count.saturating_add(1);
                        state.events.push_back(NativeImeEvent::End);
                        println!(
                            "EXCLUSIVE WM_IME_ENDCOMPOSITION count={} consumed=true",
                            state.end_count
                        );
                        return LRESULT(0);
                    }
                    _ => {}
                }
            }
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    struct ExclusiveProbeApp {
        inner: ProbeEframeApp,
        exclusive_hwnd: Option<HWND>,
    }

    impl ExclusiveProbeApp {
        fn new(runtime: ProbeRuntime) -> Self {
            Self {
                inner: ProbeEframeApp::new(runtime),
                exclusive_hwnd: None,
            }
        }

        /// Browser が active になった後だけ排他 subclass を追加する。
        /// 後から追加した subclass が先に message を受け、Composition 系 message を消費する。
        fn ensure_exclusive_subclass(&mut self) {
            if self.exclusive_hwnd.is_some() || !self.inner.browser_active {
                return;
            }
            let Some(hwnd) = self.inner.installed_hwnd else { return; };
            unsafe {
                if SetWindowSubclass(
                    hwnd,
                    Some(exclusive_native_ime_subclass_proc),
                    EXCLUSIVE_SUBCLASS_ID,
                    0,
                )
                .as_bool()
                {
                    self.exclusive_hwnd = Some(hwnd);
                    println!(
                        "Exclusive native IME -> CEF subclass installed after Browser activation: hwnd=0x{:X}",
                        hwnd.0 as usize
                    );
                } else {
                    eprintln!("Exclusive native IME -> CEF subclass installation failed");
                }
            }
        }
    }

    impl Drop for ExclusiveProbeApp {
        fn drop(&mut self) {
            if let Some(hwnd) = self.exclusive_hwnd.take() {
                unsafe {
                    let _ = RemoveWindowSubclass(
                        hwnd,
                        Some(exclusive_native_ime_subclass_proc),
                        EXCLUSIVE_SUBCLASS_ID,
                    );
                }
            }
        }
    }

    impl eframe::App for ExclusiveProbeApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            // 既存 bridge の描画・Browser click・CEF queue 転送をそのまま利用する。
            self.inner.update(ctx, frame);
            self.ensure_exclusive_subclass();
        }
    }

    pub fn run() -> eframe::Result<()> {
        let cli_args: Vec<String> = std::env::args().collect();
        if is_cef_subprocess(&cli_args) {
            std::process::exit(run_subprocess());
        }

        println!("WV-11-04-02 exclusive Windows native IME -> CEF bridge probe start");
        println!("Click Browser input once, then enable Japanese IME and type without confirming.");
        println!("Composition messages are consumed before DefSubclassProc after Browser activation.");

        let runtime = match ProbeRuntime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("exclusive native IME bridge probe failed: {error}");
                std::process::exit(1);
            }
        };

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 Exclusive Native IME -> CEF Bridge Probe")
                .with_inner_size([1100.0, 760.0]),
            ..Default::default()
        };

        eframe::run_native(
            "WV-11-04-02 Exclusive Native IME -> CEF Bridge Probe",
            native_options,
            Box::new(move |_cc| Ok(Box::new(ExclusiveProbeApp::new(runtime)))),
        )
    }
}

fn main() -> eframe::Result<()> {
    probe::run()
}
