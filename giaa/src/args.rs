use clap::Parser;
use tracing::Level;

/// 欢迎使用 GIAA (Genshin Impact Artifact Assistant) 原神圣遗物助手
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 原神窗口名称
    #[arg(short, long, default_values = ["原神", "Genshin Impact"])]
    pub window_titles: Vec<String>,

    /// 显示所有可用的窗口标题
    #[arg(long, default_value_t = false)]
    pub list_window_titles: bool,

    /// 规则文件路径
    #[arg(short, long, default_value = "rules.yaml")]
    pub rules_file: String,

    /// 日志等级 (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: Option<Level>,

    /// 日志文件路径
    #[arg(long, default_value = "giaa.log")]
    pub log_file: String,

    /// 追加日志到文件
    #[arg(long, default_value_t = false)]
    pub append_log: bool,

    /// 重置筛选条件
    #[arg(long, default_value_t = false)]
    pub reset_filter: bool,

    /// 截图识别延时时长, 电脑性能好可适当调小 (单位: 毫秒)
    #[arg(long, default_value_t = 150)]
    pub screenshot_delay: u64,

    /// 启用识别严格模式 (严格模式下: 识别圣遗物需全部属性正确才会执行动作)
    #[arg(long, default_value_t = false)]
    pub strict_mode: bool,
}

impl Args {
    /// 创建命令行参数解析器
    pub fn new() -> Self {
        Self::parse()
    }
}
