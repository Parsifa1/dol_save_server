use std::{net::SocketAddr, path::PathBuf};

const HTLP_TEMPLATE: &str = r#"
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}"#;

#[derive(Debug, clap::Parser)]
#[command(
    author,
    version,
    help_template = HTLP_TEMPLATE
)]
pub struct Args {
    /// 游戏根目录
    #[arg(long, default_value = "./")]
    pub root: PathBuf,
    /// 访问"/"时的默认文件名
    #[arg(long, default_value = "Degrees of Lewdity.html")]
    pub index: String,
    /// 服务地址
    #[arg(long, default_value = "127.0.0.1:5000")]
    pub bind: SocketAddr,
    /// 存档保存目录
    #[arg(long, default_value = "./save")]
    pub save_dir: PathBuf,
    /// 启动时跳过初始化模组流程
    #[arg(long)]
    pub no_init_mod: bool,
    /// 是否启用登录验证
    #[arg(long)]
    pub enable_auth: bool,
    /// 用户列表文件路径
    ///
    /// 文件格式应如:
    /// {
    ///     "用户名1": "密码1",
    ///     "用户名2": "密码2",
    ///     ...
    /// }
    ///
    /// 注意: 用户名应为运行系统的合法文件路径, 否则可能导致文件保存失败
    #[arg(long, default_value = "./auth.json")]
    pub auth_file: PathBuf,
}
