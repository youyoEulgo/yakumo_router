use crate::config::{AppConfig, load_config};
use notify::{EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn spawn_config_watcher(config_path: PathBuf, config: Arc<RwLock<AppConfig>>) {
    tokio::spawn(async move {
        let watch_path = config_path.clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut watcher =
            match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                let _ = tx.send(result);
            }) {
                Ok(watcher) => watcher,
                Err(e) => {
                    eprintln!("[{}] Config watcher error: {}", crate::ts(), e);
                    return;
                }
            };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
            eprintln!("[{}] Config watcher error: {}", crate::ts(), e);
            return;
        }

        while let Some(event) = rx.recv().await {
            match event {
                Ok(event) if is_reload_event(&event.kind) => match load_config(&config_path) {
                    Ok(new_config) => {
                        *config.write().await = new_config;
                        println!(
                            "[{}] Config reloaded from {}",
                            crate::ts(),
                            config_path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("[{}] Config reload failed: {}", crate::ts(), e);
                    }
                },
                Ok(_) => {}
                Err(e) => eprintln!("[{}] Config watcher error: {}", crate::ts(), e),
            }
        }
    });
}

fn is_reload_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}
