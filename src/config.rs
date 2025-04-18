use std::{
    error::Error,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// 游戏根目录
    pub root: PathBuf,
    /// 访问"/"时的默认文件名
    pub index: String,
    /// 服务地址
    pub bind: SocketAddr,
    /// 存档保存目录
    pub save_dir: PathBuf,
    /// 启动时跳过初始化模组流程
    pub init_mod: bool,
    /// 用户认证
    #[serde(default)]
    pub auth: Auth,
    /// TLS 配置
    #[serde(default)]
    pub tls: Tls,
    /// PWA 配置
    #[serde(default)]
    pub pwa: Pwa,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Auth {
    /// 是否启用
    pub enable: bool,
    /// 用户列表
    #[serde(default)]
    pub users: Vec<User>,
}

/// 认证用户信息
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Tls {
    pub enable: bool,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub cert: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Pwa {
    pub enable: bool,
    pub source: String,
}
impl Config {
    /// 默认加载的存档路径
    ///
    /// 可通过环境变量`DOL_SAVE_SERVER`修改
    pub const PATH: &str = "./dol_save_server.toml";

    /// 默认存档内容
    pub const DEFAULT: &str = include_str!("../dol_save_server.toml");

    /// 加载配置
    pub async fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Path::new(
            &std::env::var("DOL_SAVE_SERVER").unwrap_or_else(|_| Config::PATH.to_string()),
        )
        .to_path_buf();

        if !config_path.exists() {
            info!("配置文件不存在, 生成默认配置");
            tokio::fs::write(&config_path, Config::DEFAULT)
                .await
                .inspect_err(|error| error!(%error, "生成默认配置文件失败"))?;
        }

        let config = tokio::fs::read_to_string(&config_path)
            .await
            .inspect_err(|error| error!(%error, ?config_path, "读取配置文件失败"))?;

        let config = toml::from_str::<Config>(&config)
            .inspect_err(|error| error!(%error, ?config_path, "解析配置文件失败"))?;

        info!(?config, "当前配置");

        Ok(config)
    }
}

impl fmt::Debug for Tls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tls")
            .field("enable", &self.enable)
            .field("key", &"***")
            .field("cert", &"***")
            .finish()
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}
