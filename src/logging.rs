use std::{
    collections::VecDeque,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{Builder, Rotation};

const DEFAULT_MEMORY_LINES: usize = 2_000;

static MEMORY: OnceLock<Arc<Mutex<VecDeque<String>>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub directory: PathBuf,
    pub file_prefix: String,
    pub retention_days: usize,
    pub memory_lines: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("logs"),
            file_prefix: "wfide".into(),
            retention_days: 7,
            memory_lines: DEFAULT_MEMORY_LINES,
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

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_target(true)
        .with_writer(make_writer)
        .try_init()?;

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
