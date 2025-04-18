use axum::{extract::State, response::Html, Json};
use chrono::TimeZone;
use serde::Serialize;

use crate::{auth::User, Cfg};

/// 获取存档列表
#[instrument(skip(state))]
pub async fn list(State(state): State<Cfg>, User(user): User) -> Json<Vec<Save>> {
    let save_dir = state.save_dir.join(user);
    debug!(?save_dir, "存档目录");

    let mut list = vec![];
    if save_dir.exists() {
        if let Ok(mut files) = tokio::fs::read_dir(&save_dir).await {
            while let Ok(Some(file)) = files.next_entry().await {
                let path = file.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "save") {
                    let time = path
                        .metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .and_then(|d| chrono::Local.timestamp_opt(d.as_secs() as i64, 0).single())
                        .map(|time| time.format(" %F %T").to_string())
                        .unwrap_or_default();

                    let name = file.file_name().to_string_lossy().to_string();
                    list.push(Save { name, time });
                }
            }
        }
    }

    Json(list)
}

/// 独立存档页面
pub async fn page() -> Html<&'static str> {
    const TEMPLATE: &str = include_str!("../../html/savelist.html");
    Html(TEMPLATE)
}

#[derive(Debug, Serialize)]
pub struct Save {
    pub name: String,
    pub time: String,
}
