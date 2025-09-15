use anyhow::bail;
use anyhow::{Error, Result, anyhow};
use rust_decimal::prelude::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tracing::debug;

use crate::{
    ExprVar, ExprVarKey,
    parse::{Expr, parse},
};

/// 表达式结果
#[derive(Debug, PartialEq, Clone)]
pub enum ExprResult {
    Number(Decimal),
    Boolean(bool),
}

/// 表达式解析器
pub struct Parser {
    pub precision: u32,
    pub var_key: ExprVarKey,
}

impl Parser {
    /// 创建解析器
    ///
    /// # 参数
    ///
    /// * `precision` - 精度
    /// * `var_key` - 变量键
    pub fn new(precision: u32, var_key: ExprVarKey) -> Result<Self> {
        debug!("初始化解析器, 精度: {},变量: {:?}.", precision, var_key);
        Ok(Self { precision, var_key })
    }

    /// 检查表达式变量是否受支持
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式
    pub fn check_vars(&self, expr: &Expr) -> Result<()> {
        debug!("检查表达式变量: {:?}.", expr);
        match expr {
            Expr::NumberVariable(name) => {
                if !self.var_key.number_keys.contains(&name.to_string()) {
                    bail!("数字变量 '{}' 不受支持", name);
                }
            }
            Expr::BooleanVariable(name) => {
                if !self.var_key.boolean_keys.contains(&name.to_string()) {
                    bail!("布尔变量 '{}' 不受支持", name);
                }
            }
            Expr::Plus(left, right)
            | Expr::Minus(left, right)
            | Expr::Times(left, right)
            | Expr::Divide(left, right)
            | Expr::And(left, right)
            | Expr::Or(left, right)
            | Expr::Equal(left, right)
            | Expr::NotEqual(left, right)
            | Expr::LessThan(left, right)
            | Expr::GreaterThan(left, right)
            | Expr::LessThanEqual(left, right)
            | Expr::GreaterThanEqual(left, right) => {
                self.check_vars(left)?;
                self.check_vars(right)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn parse(&self, input: &str) -> Result<Expr> {
        debug!("解析表达式: '{}'.", input);
        let expr: Expr = parse(input)?;
        self.check_vars(&expr)?;
        Ok(expr)
    }

    /// 执行表达式
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式
    /// * `expr_var` - 表达式变量
    pub fn exec(&self, expr: &Expr, expr_var: &ExprVar) -> Result<ExprResult> {
        debug!(
            "执行表达式: {:#?}, 数字变量: {:?}, 布尔变量: {:?}.",
            expr, expr_var.number_vars, expr_var.boolean_vars
        );

        let err_msg = |m: &str| -> Error { anyhow!("无效的操作数类型: {}", m) };

        let arithmetic_op = |op_name: &str,
                             op: fn(Decimal, Decimal) -> Decimal,
                             left: &Expr,
                             right: &Expr|
         -> Result<ExprResult> {
            match (self.exec(left, expr_var)?, self.exec(right, expr_var)?) {
                (ExprResult::Number(l), ExprResult::Number(r)) => Ok(ExprResult::Number(op(l, r))),
                _ => Err(err_msg(op_name)),
            }
        };

        let logical_op = |op_name: &str,
                          op: fn(bool, bool) -> bool,
                          left: &Expr,
                          right: &Expr|
         -> Result<ExprResult> {
            match (self.exec(left, expr_var)?, self.exec(right, expr_var)?) {
                (ExprResult::Boolean(l), ExprResult::Boolean(r)) => {
                    Ok(ExprResult::Boolean(op(l, r)))
                }
                _ => Err(err_msg(op_name)),
            }
        };

        let comparison_op = |op_name: &str,
                             op: fn(Decimal, Decimal) -> bool,
                             left: &Expr,
                             right: &Expr|
         -> Result<ExprResult> {
            match (self.exec(left, expr_var)?, self.exec(right, expr_var)?) {
                (ExprResult::Number(l), ExprResult::Number(r)) => Ok(ExprResult::Boolean(op(
                    l.round_dp(self.precision),
                    r.round_dp(self.precision),
                ))),
                _ => Err(err_msg(op_name)),
            }
        };

        let expr_result = match expr {
            // 基础
            Expr::Number(n) => Ok(ExprResult::Number(*n)),
            Expr::Boolean(b) => Ok(ExprResult::Boolean(*b)),
            // 变量
            Expr::NumberVariable(name) => {
                if let Some(n) = expr_var.number_vars.get(name) {
                    return Ok(ExprResult::Number(Decimal::from_f32(*n).unwrap()));
                }
                Err(anyhow!("数字变量 '{}' 不受支持", name))
            }
            Expr::BooleanVariable(name) => {
                if let Some(b) = expr_var.boolean_vars.get(name) {
                    return Ok(ExprResult::Boolean(*b));
                }
                Err(anyhow!("布尔变量 '{}' 不受支持", name))
            }
            // 数字运算
            Expr::Plus(left, right) => arithmetic_op("+", |l, r| l + r, left, right),
            Expr::Minus(left, right) => arithmetic_op("-", |l, r| l - r, left, right),
            Expr::Times(left, right) => arithmetic_op("*", |l, r| l * r, left, right),
            Expr::Divide(left, right) => arithmetic_op("/", |l, r| l / r, left, right),
            // 逻辑运算
            Expr::And(left, right) => logical_op("&&", |l, r| l && r, left, right),
            Expr::Or(left, right) => logical_op("||", |l, r| l || r, left, right),
            Expr::Not(expr) => match self.exec(expr, expr_var)? {
                ExprResult::Boolean(b) => Ok(ExprResult::Boolean(!b)),
                _ => Err(err_msg("!")),
            },
            // 比较运算
            Expr::LessThan(left, right) => comparison_op("<", |l, r| l < r, left, right),
            Expr::GreaterThan(left, right) => comparison_op(">", |l, r| l > r, left, right),
            Expr::LessThanEqual(left, right) => comparison_op("<=", |l, r| l <= r, left, right),
            Expr::GreaterThanEqual(left, right) => comparison_op(">=", |l, r| l >= r, left, right),
            Expr::Equal(left, right) => {
                let left_result = self.exec(left, expr_var)?;
                let right_result = self.exec(right, expr_var)?;
                match (left_result, right_result) {
                    (ExprResult::Number(l), ExprResult::Number(r)) => Ok(ExprResult::Boolean(
                        l.round_dp(self.precision).eq(&r.round_dp(self.precision)),
                    )),
                    (ExprResult::Boolean(l), ExprResult::Boolean(r)) => {
                        Ok(ExprResult::Boolean(l == r))
                    }
                    _ => Err(err_msg("==")),
                }
            }
            Expr::NotEqual(left, right) => {
                let left_result = self.exec(left, expr_var)?;
                let right_result = self.exec(right, expr_var)?;
                match (left_result, right_result) {
                    (ExprResult::Number(l), ExprResult::Number(r)) => Ok(ExprResult::Boolean(
                        l.round_dp(self.precision).ne(&r.round_dp(self.precision)),
                    )),
                    (ExprResult::Boolean(l), ExprResult::Boolean(r)) => {
                        Ok(ExprResult::Boolean(l != r))
                    }
                    _ => Err(err_msg("!=")),
                }
            }
        };
        debug!("表达式结果: {:?}.", expr_result);
        expr_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRECISION: u32 = 2;

    #[test]
    #[should_panic(expected = "数字变量 'var' 不受支持")]
    fn test_parse_var_not_supported() {
        let input = "var>10";

        let var_key = ExprVarKey::default();

        let parser = Parser::new(PRECISION, var_key).unwrap();
        let _ = parser.parse(input).unwrap();
    }

    #[test]
    fn test_exec_number() {
        let input = "1.0+2.0==3.0";

        let var_key = ExprVarKey::default();

        let parser = Parser::new(PRECISION, var_key).unwrap();
        let expr = parser.parse(input).unwrap();

        let expr_var = ExprVar::default();
        let output = parser.exec(&expr, &expr_var).unwrap();

        assert_eq!(output, ExprResult::Boolean(true))
    }

    #[test]
    fn test_exec_number_variable() {
        let input = "a+b*c+d>10";

        let var_key = ExprVarKey::new(
            vec![],
            vec![
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("d"),
            ],
        );

        let parser = Parser::new(PRECISION, var_key).unwrap();
        let expr = parser.parse(input).unwrap();

        let mut expr_var = ExprVar::default();
        expr_var.number_vars.insert(String::from("a"), 5.0);
        expr_var.number_vars.insert(String::from("b"), 2.0);
        expr_var.number_vars.insert(String::from("c"), 3.0);
        expr_var.number_vars.insert(String::from("d"), 4.0);

        let output = parser.exec(&expr, &expr_var).unwrap();

        assert_eq!(output, ExprResult::Boolean(true))
    }

    #[test]
    fn test_exec_boolean_variable() {
        let input = "a&&b||c&&!d";

        let var_key = ExprVarKey::new(
            vec![
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("d"),
            ],
            vec![],
        );

        let parser = Parser::new(PRECISION, var_key).unwrap();
        let expr = parser.parse(input).unwrap();

        let mut expr_var = ExprVar::default();
        expr_var.boolean_vars.insert(String::from("a"), true);
        expr_var.boolean_vars.insert(String::from("b"), false);
        expr_var.boolean_vars.insert(String::from("c"), true);
        expr_var.boolean_vars.insert(String::from("d"), false);

        let output = parser.exec(&expr, &expr_var).unwrap();

        assert_eq!(output, ExprResult::Boolean(true))
    }

    #[test]
    // 测试精度
    fn test_precision() {
        let input = "10 / 3 == 3.33";

        let var_key = ExprVarKey::default();

        let parser = Parser::new(PRECISION, var_key).unwrap();
        let expr = parser.parse(input).unwrap();

        let expr_var = ExprVar::default();
        let output = parser.exec(&expr, &expr_var).unwrap();

        assert_eq!(output, ExprResult::Boolean(true));

        let input = "10 / 3 == 3.334";
        let expr = parser.parse(input).unwrap();
        let output = parser.exec(&expr, &expr_var).unwrap();

        assert_eq!(output, ExprResult::Boolean(true));
    }
}
