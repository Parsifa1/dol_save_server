#[macro_use]
extern crate tracing;

use std::{error::Error, path::Path, sync::Arc};

mod auth;
mod config;
mod pwa;
mod save;

use axum::Router;
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use config::Config;
use pwa::init_pwa;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

pub type Cfg = Arc<Config>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_log();

    let config = Config::load().await?;

    let index = config.root.join(&config.index);
    let root = config.root.clone();
    let mut app = Router::new();

    if config.init_mod {
        init_mod(&root)?;
    }

    if config.pwa.enable {
        init_pwa(&config)?;
        let sw_path = config.root.join(&config.pwa.source).join("sw.js");
        let manifest = config.root.join(&config.pwa.source).join("manifest.json");
        let icon = config.root.join(&config.pwa.source).join("icon.png");
        app = app.route_service("/sw.js", ServeFile::new(sw_path));
        app = app.route_service("/manifest.json", ServeFile::new(manifest));
        app = app.route_service("/icon.png", ServeFile::new(icon));
    }
    let cfg = Cfg::new(config);
    app = app
        // 存档相关接口
        .merge(save::router())
        // 主页
        .route_service("/", ServeFile::new(index))
        // 其他文件
        .fallback_service(ServeDir::new(root));

    if cfg.auth.enable {
        app = auth::router(app, cfg.tls.enable).await;
    }
    let app: Router<()> = app
        .layer(
            CompressionLayer::new()
                .br(true)
                .deflate(true)
                .gzip(true)
                .zstd(true),
        )
        .with_state(cfg.clone());

    let listener = std::net::TcpListener::bind(&cfg.bind)
        .inspect_err(|error| error!(%error, "监听服务地址失败"))?;

    let addr = listener.local_addr().unwrap();

    if cfg.tls.enable {
        let tls = RustlsConfig::from_pem(
            cfg.tls.cert.clone().into_bytes(),
            cfg.tls.key.clone().into_bytes(),
        )
        .await
        .inspect_err(|error| error!(%error, "初始化TLS配置失败"))?;

        info!("服务地址: https://{addr}/");
        info!("你可以访问 https://{addr}/saves 来查看服务端已保存的存档");

        let acceptor = RustlsAcceptor::new(tls);
        axum_server::from_tcp(listener)
            .acceptor(acceptor)
            .serve(app.into_make_service())
            .await
            .inspect_err(|error| error!(%error, "服务启动失败"))?;
    } else {
        info!("服务地址: http://{addr}/");
        info!("你可以访问 http://{addr}/saves 来查看服务端已保存的存档");

        axum_server::from_tcp(listener)
            .serve(app.into_make_service())
            .await
            .inspect_err(|error| error!(%error, "服务启动失败"))?;
    }

    Ok(())
}

fn init_log() {
    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::rfc_3339())
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!("{}=info,warn", env!("CARGO_CRATE_NAME")))
            }),
        )
        .init();
}

#[instrument]
pub fn init_mod(dir: &Path) -> std::io::Result<()> {
    const MOD_LIST_NAME: &str = "modList.json";

    //all mods
    const SERVER_MOD_DATA: &[u8] = include_bytes!("../mod/save_server.mod.zip");
    const DOLSIMS_MOD_DATA: &[u8] = include_bytes!("../mod/DoLSims.mod.zip");

    info!("开始初始化模组");
    let mod_list_path = dir.join(MOD_LIST_NAME);
    let mod_list = {
        let s = std::fs::read_to_string(&mod_list_path)
            .inspect_err(|error| error!(%error, ?mod_list_path, "读取模组列表败"))?;
        serde_json::from_str::<Vec<String>>(&s)
            .inspect_err(|error| error!(%error, ?mod_list_path, "反序列化模组列表失败"))?
    };

    for mod_name in mod_list {
        let mod_path = dir.join(&mod_name);
        let data = match mod_name.as_str().strip_prefix("mod/").unwrap_or(&mod_name) {
            "save_server.mod.zip" => SERVER_MOD_DATA,
            "DoLSims.mod.zip" => DOLSIMS_MOD_DATA,
            //more mods...
            _ => {
                error!(%mod_name, "未知模组");
                continue;
            }
        };
        std::fs::write(&mod_path, data)
            .inspect_err(|error| error!(%error, ?mod_path,"保存{}失败", mod_name))?;
        info!(?mod_path, "已保存{}", mod_name);
    }

    info!("模组初始化结束");

    Ok(())
}
