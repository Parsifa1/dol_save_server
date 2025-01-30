use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::auth::User;

#[derive(Debug, Deserialize)]
pub struct Save {
    pub slot: u32,
    pub name: String,
    pub data: String,
    pub new: bool,
}

#[instrument(skip(data, state))]
pub async fn save(
    State(state): State<crate::State>,
    User(user): User,
    Json(Save {
        slot,
        name,
        data,
        new,
    }): Json<Save>,
) -> (StatusCode, Json<&'static str>) {
    let save_dir = state.save_dir.join(user);
    debug!(?save_dir, "存档目录");

    if let Err(error) = tokio::fs::create_dir_all(&save_dir).await {
        const MSG: &str = "创建存档目录失败";
        warn!(%error, ?save_dir, "{MSG}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(MSG));
    }

    let file_name = format!("{}-{name}-{slot:02}.save", if new { "new" } else { "old" });
    let save_path = save_dir.join(file_name);
    if let Err(error) = tokio::fs::write(&save_path, data).await {
        const MSG: &str = "存档文件保存失败";
        warn!(%error, ?save_path, "{MSG}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(MSG));
    }

    const MSG: &str = "存档保存成功";
    info!(?save_path, "{MSG}");

    (StatusCode::OK, Json(MSG))
}
