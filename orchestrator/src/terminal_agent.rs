use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// A mounted directory session — the AI's working context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub content: String,
    pub size: usize,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteResult {
    pub bytes_written: usize,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: String,  // "file" or "directory"
    pub size: usize,
}

pub struct TerminalAgent {
    pub sessions: RwLock<HashMap<String, TerminalSession>>,
    pub next_id: RwLock<u64>,
}

impl TerminalAgent {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }

    pub async fn mount(&self, path: &str) -> Result<TerminalSession, String> {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Err(format!("Path does not exist: {}", path));
        }
        if !path_buf.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // Resolve to absolute path
        let absolute = std::fs::canonicalize(&path_buf)
            .map_err(|e| format!("Failed to resolve path: {}", e))?;

        let id = {
            let mut counter = self.next_id.write().await;
            let id = format!("term_{}", *counter);
            *counter += 1;
            id
        };

        let session = TerminalSession {
            id: id.clone(),
            path: absolute.to_string_lossy().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.sessions.write().await.insert(id, session.clone());
        Ok(session)
    }

    pub async fn unmount(&self, session_id: &str) -> Result<(), String> {
        self.sessions
            .write()
            .await
            .remove(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        Ok(())
    }

    pub async fn exec(&self, session_id: &str, command: &str) -> Result<ExecResult, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or("Session not found")?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&session.path)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        Ok(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    pub async fn read_file(&self, session_id: &str, file_path: &str) -> Result<FileContent, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or("Session not found")?;

        let full_path = Path::new(&session.path).join(file_path);
        let full_path = full_path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;

        // Security: ensure the resolved path is within the session directory
        if !full_path.starts_with(&session.path) {
            return Err("Access denied: path escapes session directory".to_string());
        }

        let metadata = tokio::fs::metadata(&full_path)
            .await
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        if metadata.is_dir() {
            return Ok(FileContent {
                content: String::new(),
                size: 0,
                is_directory: true,
            });
        }

        let content = tokio::fs::read_to_string(&full_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(FileContent {
            content,
            size: metadata.len() as usize,
            is_directory: false,
        })
    }

    pub async fn write_file(&self, session_id: &str, file_path: &str, content: &str) -> Result<WriteResult, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or("Session not found")?;

        let full_path = Path::new(&session.path).join(file_path);
        let full_path = full_path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;

        // Security: ensure the resolved path is within the session directory
        if !full_path.starts_with(&session.path) {
            return Err("Access denied: path escapes session directory".to_string());
        }

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        tokio::fs::write(&full_path, content)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(WriteResult {
            bytes_written: content.len(),
            path: file_path.to_string(),
        })
    }

    pub async fn list_dir(&self, session_id: &str, dir_path: &str) -> Result<Vec<DirEntry>, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or("Session not found")?;

        let full_path = Path::new(&session.path).join(dir_path);
        let full_path = full_path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;

        // Security: ensure the resolved path is within the session directory
        if !full_path.starts_with(&session.path) {
            return Err("Access denied: path escapes session directory".to_string());
        }

        if !full_path.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&full_path)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
            let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }
            entries.push(DirEntry {
                name,
                entry_type: if metadata.is_dir() { "directory".to_string() } else { "file".to_string() },
                size: metadata.len() as usize,
            });
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            match (&a.entry_type, &b.entry_type) {
                (t1, t2) if t1 == "directory" && t2 != "directory" => std::cmp::Ordering::Less,
                (t1, t2) if t1 != "directory" && t2 == "directory" => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(entries)
    }

    pub async fn list_sessions(&self) -> Vec<TerminalSession> {
        self.sessions.read().await.values().cloned().collect()
    }
}
