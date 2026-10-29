use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, RunEvent};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

#[derive(Default)]
struct BackendProcess(Mutex<Option<CommandChild>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let legacy_database = find_legacy_database();
            let data_dir = if cfg!(debug_assertions) {
                legacy_database
                    .as_ref()
                    .and_then(|path| path.parent())
                    .map(PathBuf::from)
                    .unwrap_or(app_data_dir)
            } else {
                app_data_dir
            };
            std::fs::create_dir_all(&data_dir)?;

            let mut backend_child = None;
            if !backend_is_ready() {
                let mut command = app
                    .shell()
                    .sidecar("grid-metropolis-server")?
                    .env("PORT", "3000")
                    .env("HOST", "127.0.0.1")
                    .env("GRID_METROPOLIS_DATA_DIR", data_dir.to_string_lossy().to_string());

                if !cfg!(debug_assertions) {
                    if let Some(legacy_path) = legacy_database.as_ref() {
                        command = command.env("GRID_METROPOLIS_LEGACY_DB", legacy_path.to_string_lossy().to_string());
                    }
                }

                match command.spawn() {
                    Ok((mut events, child)) => {
                        backend_child = Some(child);
                        tauri::async_runtime::spawn(async move {
                            while let Some(event) = events.recv().await {
                                match event {
                                    CommandEvent::Error(error) => eprintln!("Falha no backend local: {error}"),
                                    CommandEvent::Terminated(payload) => {
                                        eprintln!("Backend local encerrado: {payload:?}");
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    Err(error) => eprintln!("Não foi possível iniciar o backend incluído: {error}"),
                }
            }

            app.manage(BackendProcess(Mutex::new(backend_child)));
            Ok(())
        })
        .build(tauri::generate_context!());

    match result {
        Ok(app) => app.run(|app_handle, event| {
            if let RunEvent::Exit = event {
                let state = app_handle.state::<BackendProcess>();
                let child = state.0.lock().ok().and_then(|mut child| child.take());
                if let Some(child) = child {
                    if let Err(error) = child.kill() {
                        eprintln!("Falha ao encerrar o backend local: {error}");
                    }
                }
            }
        }),
        Err(error) => eprintln!("Falha ao iniciar The Grid Metropolis: {error}"),
    }
}

fn backend_is_ready() -> bool {
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(400)) else {
        return false;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(600)));
    let request = b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }

    response.starts_with("HTTP/1.1 200") && response.contains("the-grid-metropolis-bridge")
}

fn find_legacy_database() -> Option<PathBuf> {
    let current_directory = std::env::current_dir().ok()?;
    let candidates = [
        current_directory.join("../server/data/grid.db"),
        current_directory.join("../data/grid.db"),
        current_directory.join("server/data/grid.db"),
        current_directory.join("data/grid.db"),
    ];

    candidates.into_iter().find(|path| path.is_file())
}
