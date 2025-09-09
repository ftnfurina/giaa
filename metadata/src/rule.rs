use std::fs;

use anyhow::Context;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 操作动作
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    #[schemars(title = "点击锁定")]
    #[serde(rename = "点击锁定")]
    ClickLock,
    #[schemars(title = "点击标记")]
    #[serde(rename = "点击标记")]
    ClickMark,
    #[schemars(title = "锁定")]
    #[serde(rename = "锁定")]
    Lock,
    #[schemars(title = "仅锁定")]
    #[serde(rename = "仅锁定")]
    OnlyLock,
    #[schemars(title = "锁定和标记")]
    #[serde(rename = "锁定和标记")]
    LockAndMark,
    #[schemars(title = "取消锁定和标记")]
    #[serde(rename = "取消锁定和标记")]
    UnLockAndMark,
}

/// 规则
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    #[schemars(title = "规则描述")]
    pub description: String,
    #[schemars(title = "规则表达式")]
    pub expression: String,
    #[schemars(title = "表达式命中后执行操作")]
    pub action: RuleAction,
}

impl Rule {
    /// 通过文件名加载规则
    ///
    /// # 参数
    ///
    /// * `rules_file` - 规则文件名
    pub fn load(rules_file: &str) -> Result<Vec<Rule>> {
        let rules_data = fs::read(&rules_file).context("读取规则文件失败")?;
        let rules = serde_yaml::from_slice::<Vec<Rule>>(rules_data.as_slice())
            .context("解析规则文件失败, 请检查格式是否正确")?;
        Ok(rules)
    }
}
