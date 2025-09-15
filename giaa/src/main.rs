use std::path::Path;

use anyhow::{Result, bail};
use metadata::ARTIFACT_INFO;
use metadata::{Coordinate, Rule};
use ocr::PPOcr;
use parser::ExprVarKey;
use parser::Parser;
use std::io::stdin;
use tracing::{error, info};
use window::WinWindow;
use window::Window;

use crate::identifier::ArtifactIdentify;
use crate::log::init_log;
use crate::rule_expr::RuleExpr;
use crate::{
    actuator::Actuator, args::Args, converter::Converter, identifier::Identifier, scanner::Scanner,
};

mod actuator;
mod args;
mod artifact;
mod color;
mod converter;
mod error;
mod identifier;
mod log;
mod rule_expr;
mod scanner;

/// 程序入口
fn application() -> Result<()> {
    let args = Args::new();

    init_log(&args)?;

    info!("欢迎使用 GIAA (Genshin Impact Artifact Assistant) 原神圣遗物助手");

    if args.list_window_titles {
        info!("可用的窗口标题:");
        for title in WinWindow::list_window_titles()? {
            info!("  - {}", title);
        }
        return Ok(());
    }

    if !Path::new(&args.rules_file).exists() {
        bail!("规则文件 {} 不存在", args.rules_file);
    }
    let rules = Rule::load(&args.rules_file)?;
    if rules.is_empty() {
        bail!("规则文件 {} 为空, 请添加规则内容", args.rules_file);
    }

    let var_key = ExprVarKey::new(
        ARTIFACT_INFO.get_boolean_keys(),
        ARTIFACT_INFO.get_number_keys(),
    );

    // 表达式解析器
    let parser = Parser::new(3, var_key)?;

    // 规则解析
    let rule_exprs = RuleExpr::from_rules(&rules, &parser)?;
    // 圣遗物属性识别筛选
    let artifact_identify = ArtifactIdentify::filter(&rule_exprs)?;
    // OCR 识别
    let pp_ocr = PPOcr::new()?;

    // 窗口管理
    let win_window = WinWindow::new(&args.window_titles)?;
    win_window.try_focus()?;
    let window_rect = win_window.rect()?;

    // 获取当前环境适配坐标数据
    let coordinate = Coordinate::load(window_rect.1)?;
    // 坐标转换器
    let converter = Converter::new(&coordinate.resolution, window_rect)?;

    // 圣遗物识别器
    let identifier = Identifier::new(
        &converter,
        &pp_ocr,
        &coordinate.data,
        &artifact_identify,
        &args,
    )?;
    // 动作执行器
    let actuator = Actuator::new(&parser, &win_window, &converter, &rule_exprs, &coordinate)?;
    // 圣遗物扫描器
    let mut scanner = Scanner::new(
        &converter,
        &coordinate.data,
        &identifier,
        &actuator,
        &pp_ocr,
        &win_window,
        &args,
    )?;
    // 开始扫描
    scanner.scan()
}

/// 等待用户输入
fn wait_for_key_press() {
    let mut input = String::new();
    println!("按任意键退出程序...");
    stdin().read_line(&mut input).expect("读取输入失败");
}

fn main() {
    match application() {
        Ok(_) => {
            info!("程序已执行完毕");
            wait_for_key_press();
        }
        Err(e) => {
            error!("程序存在异常: {}", e);
            wait_for_key_press();
            std::process::exit(1);
        }
    }
}
