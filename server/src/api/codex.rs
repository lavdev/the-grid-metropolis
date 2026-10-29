use axum::{
    extract::Json,
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexStatusResponse {
    pub installed: bool,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub auth_exists: bool,
    pub auth_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodexExecRequest {
    pub prompt: String,
    pub working_dir: Option<String>,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub thread_id: Option<String>,
}

/// Localiza o caminho absoluto do binário oficial codex.exe no sistema
pub fn find_codex_bin() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("CODEX_BIN") {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }

    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let p = Path::new(&local_app_data)
                .join("Programs")
                .join("OpenAI")
                .join("Codex")
                .join("bin")
                .join("codex.exe");
            if p.exists() {
                return Some(p);
            }
        }
    }

    // Procura no PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            #[cfg(windows)]
            let target = dir.join("codex.exe");
            #[cfg(not(windows))]
            let target = dir.join("codex");

            if target.is_file() {
                return Some(target);
            }
        }
    }

    None
}

/// Localiza o arquivo de autenticação local ~/.codex/auth.json
pub fn find_codex_auth() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("CODEX_HOME") {
        let p = PathBuf::from(home).join("auth.json");
        if p.exists() {
            return Some(p);
        }
    }

    #[cfg(windows)]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let p = Path::new(&userprofile).join(".codex").join("auth.json");
            if p.exists() {
                return Some(p);
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let p = Path::new(&home).join(".codex").join("auth.json");
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

/// GET /api/v1/codex/status
pub async fn codex_status_handler() -> Json<CodexStatusResponse> {
    let bin = find_codex_bin();
    let auth = find_codex_auth();

    let version = if let Some(ref b) = bin {
        match std::process::Command::new(b).arg("--version").output() {
            Ok(out) => {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if v.is_empty() { None } else { Some(v) }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    Json(CodexStatusResponse {
        installed: bin.is_some(),
        version,
        executable_path: bin.map(|p| p.to_string_lossy().to_string()),
        auth_exists: auth.is_some(),
        auth_path: auth.map(|p| p.to_string_lossy().to_string()),
    })
}

/// POST /api/v1/codex/stream
pub async fn codex_stream_handler(
    Json(payload): Json<CodexExecRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, Response> {
    let bin = find_codex_bin().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Codex CLI não encontrado no sistema"})),
        )
            .into_response()
    })?;

    let mut cmd = Command::new(bin);

    // Configura o diretório raiz de trabalho (-C) antes dos subcomandos
    if let Some(ref dir) = payload.working_dir {
        if !dir.is_empty() && Path::new(dir).exists() {
            cmd.arg("-C").arg(dir);
        }
    }

    let is_resuming = payload
        .thread_id
        .as_ref()
        .map(|tid| !tid.trim().is_empty())
        .unwrap_or(false);

    if is_resuming {
        cmd.arg("exec").arg("resume");
    } else {
        cmd.arg("exec");
    }

    cmd.arg("--skip-git-repo-check")
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("--json");

    if let Some(ref m) = payload.model {
        if !m.is_empty() && m != "default" {
            cmd.arg("-m").arg(m);
        }
    }

    if let Some(ref r) = payload.reasoning_effort {
        if !r.is_empty() && r != "default" {
            cmd.arg("-c")
                .arg(format!("model_reasoning_effort=\"{}\"", r));
        }
    }

    if is_resuming {
        if let Some(ref tid) = payload.thread_id {
            cmd.arg(tid.trim());
        }
    }

    cmd.arg(&payload.prompt);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        error!("Erro ao iniciar processo do Codex CLI: {:?}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Falha ao spawnar codex: {}", e)})),
        )
            .into_response()
    })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Falha ao capturar stdout do Codex"})),
        )
            .into_response()
    })?;

    let stderr = child.stderr.take();

    let stream = async_stream::stream! {
        let mut lines = BufReader::new(stdout).lines();
        let mut emitted_any = false;

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            emitted_any = true;
            yield Ok(Event::default().data(trimmed));
        }

        let status = child.wait().await;
        if let Ok(exit_status) = status {
            if !exit_status.success() && !emitted_any {
                let mut err_str = String::new();
                if let Some(err_reader) = stderr {
                    let mut reader = BufReader::new(err_reader).lines();
                    while let Ok(Some(err_line)) = reader.next_line().await {
                        err_str.push_str(&err_line);
                        err_str.push(' ');
                    }
                }
                error!("Codex CLI falhou com código {:?}: {}", exit_status.code(), err_str);
                let err_payload = json!({
                    "type": "item.completed",
                    "item": {
                        "type": "agent_message",
                        "text": format!("⚠️ [ERRO CODEX CLI]: O processo foi finalizado com erro: {}", err_str.trim())
                    }
                });
                yield Ok(Event::default().data(err_payload.to_string()));
            }
        }

        yield Ok(Event::default().data("[DONE]"));
    };

    Ok(Sse::new(stream))
}
