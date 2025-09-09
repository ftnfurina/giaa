use anyhow::{Result, anyhow};
use metadata::Rule;
use parser::{Expr, ExprVarKey, Parser};

/// 规则与表达式映射
#[derive(Debug, Clone)]
pub struct RuleExpr {
    pub rule: Rule,
    pub expr: Expr,
    pub expr_var_key: ExprVarKey,
}

impl RuleExpr {
    /// 构造规则与表达式映射
    ///
    /// # 参数
    ///
    /// * `rule` - 规则
    /// * `parser` - 表达式解析器
    pub fn from_rule(rule: Rule, parser: &Parser) -> Result<Self> {
        let expr = parser
            .parse(&rule.expression)
            .map_err(|e| anyhow!("解析规则表达式失败: \n{}\n错误原因: {}", rule.expression, e))?;
        let expr_var_key = expr.get_var_keys();
        Ok(Self {
            rule,
            expr,
            expr_var_key,
        })
    }

    /// 批量构造规则与表达式映射
    ///
    /// # 参数
    ///
    /// * `rules` - 规则列表
    /// * `parser` - 表达式解析器
    pub fn from_rules(rules: &[Rule], parser: &Parser) -> Result<Vec<Self>> {
        rules
            .iter()
            .map(|rule| Self::from_rule(rule.clone(), parser))
            .collect()
    }
}
