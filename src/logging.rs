use std::{
    collections::VecDeque,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, reload, util::SubscriberInitExt};

const DEFAULT_MEMORY_LINES: usize = 2_000;

static MEMORY: OnceLock<Arc<Mutex<VecDeque<String>>>> = OnceLock::new();
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
    pub fn snapshot(&self) -> Vec<String> {
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

pub fn snapshot() -> Vec<String> {
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
        memory: MemoryWriter {
            buffer: memory.clone(),
            capacity: memory_lines,
            pending: Vec::new(),
        },
    };

    let (level_filter, level_handle) = reload::Layer::new(config.level.filter());
    // Keep the shared file/memory stream free of terminal ANSI escapes.
    // Console coloring can be added as a separate presentation layer later;
    // Log Viewer must receive structured/plain data rather than terminal control codes.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(make_writer);

    tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer)
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
    memory: MemoryWriter,
}

impl Write for CombinedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.console.write_all(buf)?;
        self.file.write_all(buf)?;
        self.memory.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.console.flush()?;
        self.file.flush()?;
        self.memory.flush()
    }
}

struct MemoryWriter {
    buffer: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
    pending: Vec<u8>,
}

impl MemoryWriter {
    fn push_complete_lines(&mut self) {
        while let Some(pos) = self.pending.iter().position(|byte| *byte == b'\n') {
            let bytes: Vec<u8> = self.pending.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&bytes).trim_end().to_owned();
            if let Ok(mut buffer) = self.buffer.lock() {
                while buffer.len() >= self.capacity {
                    buffer.pop_front();
                }
                buffer.push_back(line);
            }
        }
    }
}

impl Write for MemoryWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buf);
        self.push_complete_lines();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.push_complete_lines();
        Ok(())
    }
}
