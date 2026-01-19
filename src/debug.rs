//! Debug logging for MCP messages.

use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Debug logger for MCP messages.
pub struct DebugLogger {
    enabled: bool,
    trace_file: Option<Mutex<File>>,
    trace_path: Option<PathBuf>,
}

impl DebugLogger {
    /// Create a new debug logger.
    pub fn new(enabled: bool) -> Self {
        let (trace_file, trace_path) = if enabled {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let path = PathBuf::from(format!("/tmp/sap_calm_mcp_trace_{}.log", timestamp));
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
            {
                Ok(file) => {
                    eprintln!("[DEBUG] Trace file: {}", path.display());
                    (Some(Mutex::new(file)), Some(path))
                }
                Err(e) => {
                    eprintln!("[DEBUG] Failed to create trace file: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        Self {
            enabled,
            trace_file,
            trace_path,
        }
    }

    /// Get the trace file path.
    pub fn trace_path(&self) -> Option<&PathBuf> {
        self.trace_path.as_ref()
    }

    /// Log a message to stderr and trace file.
    pub fn log(&self, message: &str) {
        if !self.enabled {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let formatted = format!("[{}] {}", timestamp, message);

        eprintln!("{}", formatted);

        if let Some(ref file) = self.trace_file {
            if let Ok(mut f) = file.lock() {
                let _ = writeln!(f, "{}", formatted);
                let _ = f.flush();
            }
        }
    }

    /// Log a tool call.
    pub fn log_tool_call(&self, tool_name: &str, params: &serde_json::Value) {
        if !self.enabled {
            return;
        }

        self.log(&format!(
            "TOOL CALL: {} | params: {}",
            tool_name,
            truncate_json(params, 1000)
        ));
    }

    /// Log a tool result.
    pub fn log_tool_result(&self, tool_name: &str, result: &serde_json::Value) {
        if !self.enabled {
            return;
        }

        self.log(&format!(
            "TOOL RESULT: {} | result: {}",
            tool_name,
            truncate_json(result, 1000)
        ));
    }
}

impl std::fmt::Debug for DebugLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugLogger")
            .field("enabled", &self.enabled)
            .field("trace_path", &self.trace_path)
            .finish()
    }
}

/// Truncate a JSON value to a maximum length.
fn truncate_json(value: &serde_json::Value, max_len: usize) -> String {
    let s = value.to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("{}...(truncated)", &s[..max_len])
    }
}
