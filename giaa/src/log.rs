use std::fs::OpenOptions;

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{Layer, fmt, registry};

use crate::args::Args;

/// 初始化日志记录器
///
/// # 参数
///
/// * `args` - 命令行参数
pub fn init_log(args: &Args) -> Result<()> {
    let log_level = args.log_level.unwrap();

    let filter = Targets::new()
        .with_default(LevelFilter::from_level(log_level))
        .with_target("ort", LevelFilter::ERROR);

    let console_layer = fmt::layer()
        .with_ansi(true)
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ))
        .with_filter(filter.clone());

    let file_writer = OpenOptions::new()
        .write(true)
        .append(args.append_log)
        .create(true)
        .open(args.log_file.clone())?;
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ))
        .with_filter(filter.clone());

    let subscriber = registry().with(console_layer).with(file_layer);
    tracing::subscriber::set_global_default(subscriber).expect("设置全局日志记录器失败");

    Ok(())
}
