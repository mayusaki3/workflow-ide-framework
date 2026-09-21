use std::{
    collections::VecDeque,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{Builder, Rotation};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    filter::LevelFilter,
    layer::{Context, Layer, SubscriberExt},
    reload,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

const DEFAULT_MEMORY_LINES: usize = 2_000;

static MEMORY: OnceLock<Arc<Mutex<VecDeque<LogEntry>>>> = OnceLock::new();
static LEVEL_RELOAD: OnceLock<reload::Handle<LevelFilter, tracing_subscriber::Registry>> = OnceLock::new();
static CURRENT_LEVEL: OnceLock<Mutex<LogLevel>> = OnceLock::new();
static LEVEL_CHANGE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub level: LogLevel,
    pub target: String,
    pub message: Option<String>,
    pub fields: Vec<(String, String)>,
}

impl LogEntry {
    pub fn display_text(&self) -> String {
        let mut text = format!("{} {:?} {}", self.time, self.level, self.target);
        if let Some(message) = &self.message {
            text.push_str(": ");
            text.push_str(message);
        }
        for (name, value) in &self.fields {
            text.push(' ');
            text.push_str(name);
            text.push('=');
            text.push_str(value);
        }
        text
    }
}

#[derive(Default)]
struct EventVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl tracing::field::Visit for EventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let value = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(value.trim_matches('"').to_owned());
        } else {
            self.fields.push((field.name().to_owned(), value));
        }
    }
}

struct MemoryLayer {
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    capacity: usize,
}

impl<S> Layer<S> for MemoryLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        };
        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let seconds = now.as_secs() % 86_400;
        let millis = now.subsec_millis();
        let time = format!(
            "{:02}:{:02}:{:02}.{:03}",
            seconds / 3_600,
            (seconds % 3_600) / 60,
            seconds % 60,
            millis
        );
        // Preserve the original target for events bridged from the log crate,
        // while keeping bridge source metadata out of the normal viewer.
        let mut target = metadata.target().to_owned();
        let mut fields = Vec::new();
        for (name, value) in visitor.fields {
            if name == "log.target" {
                target = value.trim_matches('"').to_owned();
            } else if !matches!(
                name.as_str(),
                "log.module_path" | "log.file" | "log.line"
            ) {
                fields.push((name, value));
            }
        }

        let entry = LogEntry {
            time,
            level,
            target,
            message: visitor.message,
            fields,
        };
        if let Ok(mut buffer) = self.buffer.lock() {
            while buffer.len() >= self.capacity {
                buffer.pop_front();
            }
            buffer.push_back(entry);
        }
    }
}

impl LogLevel {
    fn filter(self) -> LevelFilter {
        match self {
            Self::Error => LevelFilter::ERROR,
            Self::Warn => LevelFilter::WARN,
            Self::Info => LevelFilter::INFO,
            Self::Debug => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub directory: PathBuf,
    pub file_prefix: String,
    pub retention_days: usize,
    pub memory_lines: usize,
    pub level: LogLevel,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("logs"),
            file_prefix: "wfide".into(),
            retention_days: 7,
            memory_lines: DEFAULT_MEMORY_LINES,
            level: LogLevel::Info,
        }
    }
}

pub struct LoggingGuard {
    _file_guard: WorkerGuard,
}

impl LoggingGuard {
    pub fn snapshot(&self) -> Vec<LogEntry> {
        snapshot()
    }
}

pub fn level() -> LogLevel {
    CURRENT_LEVEL
        .get()
        .and_then(|level| level.lock().ok().map(|level| *level))
        .unwrap_or(LogLevel::Info)
}

pub fn set_level(level: LogLevel) -> Result<(), &'static str> {
    let _change_guard = LEVEL_CHANGE_LOCK
        .lock()
        .map_err(|_| "WFIDE log level change lock is poisoned")?;
    let handle = LEVEL_RELOAD.get().ok_or("WFIDE logging is not initialized")?;
    let previous = self::level();

    if previous == level {
        return Ok(());
    }

    // Log-level changes are control-plane events and are always recorded as INFO.
    // Temporarily enable INFO when the current filter is WARN/ERROR, emit the
    // transition, then apply the requested level while holding the change lock.
    if matches!(previous, LogLevel::Error | LogLevel::Warn) {
        handle.reload(LevelFilter::INFO)
            .map_err(|_| "failed to temporarily enable INFO for WFIDE log level change")?;
    }

    tracing::info!(
        target: "wfide::logging",
        from = ?previous,
        to = ?level,
        "runtime log level changed"
    );

    handle.reload(level.filter()).map_err(|_| "failed to reload WFIDE log level")?;
    if let Some(current) = CURRENT_LEVEL.get() {
        if let Ok(mut current) = current.lock() {
            *current = level;
        }
    }

    Ok(())
}

pub fn snapshot() -> Vec<LogEntry> {
    MEMORY
        .get()
        .and_then(|memory| memory.lock().ok().map(|lines| lines.iter().cloned().collect()))
        .unwrap_or_default()
}

pub fn init(
    application_id: &str,
    config: &LoggingConfig,
) -> Result<LoggingGuard, Box<dyn std::error::Error + Send + Sync>> {
    fs::create_dir_all(&config.directory)?;

    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix(format!("{}-{}", config.file_prefix, application_id))
        .filename_suffix("log")
        .max_log_files(config.retention_days.max(1))
        .build(&config.directory)?;

    let (file_writer, file_guard) = tracing_appender::non_blocking(appender);
    let memory = MEMORY
        .get_or_init(|| Arc::new(Mutex::new(VecDeque::new())))
        .clone();

    let memory_lines = config.memory_lines.max(1);
    let make_writer = move || CombinedWriter {
        console: io::stdout(),
        file: file_writer.clone(),
     };

    let (level_filter, level_handle) = reload::Layer::new(config.level.filter());
    // Keep the shared file/memory stream free of terminal ANSI escapes.
    // Console coloring can be added as a separate presentation layer later;
    // Log Viewer must receive structured/plain data rather than terminal control codes.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(make_writer)
        // The in-memory Log Viewer is line-oriented. Prevent pretty/multiline
        // event formatting from producing continuation rows without metadata.
        .compact();

    let memory_layer = MemoryLayer {
        buffer: memory,
        capacity: memory_lines,
    };

    tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer)
        .with(memory_layer)
        .try_init()?;

    let _ = LEVEL_RELOAD.set(level_handle);
    let _ = CURRENT_LEVEL.set(Mutex::new(config.level));

    Ok(LoggingGuard {
        _file_guard: file_guard,
    })
}

struct CombinedWriter {
    console: io::Stdout,
    file: NonBlocking,
 }

impl Write for CombinedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.console.write_all(buf)?;
        self.file.write_all(buf)?;
         Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.console.flush()?;
        self.file.flush()?;
        Ok(())
    }
}

