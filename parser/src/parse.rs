use std::collections::HashMap;

use anyhow::Result;
use rust_decimal::prelude::*;

/// 表达式变量
#[derive(Debug, Clone)]
pub struct ExprVar {
    pub boolean_vars: HashMap<String, bool>,
    pub number_vars: HashMap<String, f32>,
}

impl ExprVar {
    /// 创建默认的表达式变量
    pub fn default() -> Self {
        Self {
            boolean_vars: HashMap::new(),
            number_vars: HashMap::new(),
        }
    }
}

/// 表达式变量键
#[derive(Debug, Clone)]
pub struct ExprVarKey {
    pub boolean_keys: Vec<String>,
    pub number_keys: Vec<String>,
}

impl ExprVarKey {
    pub fn new(boolean_keys: Vec<String>, number_keys: Vec<String>) -> Self {
        Self {
            boolean_keys,
            number_keys,
        }
    }

    pub fn default() -> Self {
        Self {
            boolean_keys: vec![],
            number_keys: vec![],
        }
    }
}

/// 表达式类型
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // 基础
    Number(Decimal), // 数字
    Boolean(bool),   // 布尔值

    // 变量
    NumberVariable(String),  // 数字变量
    BooleanVariable(String), // 布尔变量

    // 运算符
    Plus(Box<Expr>, Box<Expr>),   // 加法
    Minus(Box<Expr>, Box<Expr>),  // 减法
    Times(Box<Expr>, Box<Expr>),  // 乘法
    Divide(Box<Expr>, Box<Expr>), // 除法

    // 逻辑运算符
    And(Box<Expr>, Box<Expr>), // 与
    Or(Box<Expr>, Box<Expr>),  // 或
    Not(Box<Expr>),            // 取反

    // 比较运算符
    Equal(Box<Expr>, Box<Expr>),            // 等于
    NotEqual(Box<Expr>, Box<Expr>),         // 不等于
    LessThan(Box<Expr>, Box<Expr>),         // 小于
    GreaterThan(Box<Expr>, Box<Expr>),      // 大于
    LessThanEqual(Box<Expr>, Box<Expr>),    // 小于等于
    GreaterThanEqual(Box<Expr>, Box<Expr>), // 大于等于
}

impl Expr {
    pub fn get_var_keys(&self) -> ExprVarKey {
        loop_var_keys(self)
    }
}

/// 遍历表达式变量键
///
/// # 参数
///
/// * `expr` - 表达式
pub fn loop_var_keys(expr: &Expr) -> ExprVarKey {
    let mut boolean_keys = vec![];
    let mut number_keys = vec![];

    let mut keys_append = |left: &Expr, right: &Expr| {
        let left = loop_var_keys(left);
        let right = loop_var_keys(right);
        boolean_keys.extend(left.boolean_keys);
        boolean_keys.extend(right.boolean_keys);
        number_keys.extend(left.number_keys);
        number_keys.extend(right.number_keys);
    };

    match expr {
        Expr::NumberVariable(key) => number_keys.push(key.clone()),
        Expr::BooleanVariable(key) => boolean_keys.push(key.clone()),
        Expr::Plus(left, right) => keys_append(left, right),
        Expr::Minus(left, right) => keys_append(left, right),
        Expr::Times(left, right) => keys_append(left, right),
        Expr::Divide(left, right) => keys_append(left, right),
        Expr::And(left, right) => keys_append(left, right),
        Expr::Or(left, right) => keys_append(left, right),
        Expr::Not(expr) => {
            let var_key = loop_var_keys(expr);
            boolean_keys.extend(var_key.boolean_keys);
            number_keys.extend(var_key.number_keys);
        }
        Expr::Equal(left, right) => keys_append(left, right),
        Expr::NotEqual(left, right) => keys_append(left, right),
        Expr::LessThan(left, right) => keys_append(left, right),
        Expr::GreaterThan(left, right) => keys_append(left, right),
        Expr::LessThanEqual(left, right) => keys_append(left, right),
        Expr::GreaterThanEqual(left, right) => keys_append(left, right),
        Expr::Boolean(_) | Expr::Number(_) => (),
    }
    ExprVarKey {
        boolean_keys,
        number_keys,
    }
}

// 布尔表达式解析器
peg::parser!(grammar bool_parser() for str {
    rule _ = quiet!{[' ' | '\t' | '\n' | '\r']*}

    // 基础匹配
    rule world() = ['a'..='z' | 'A'..='Z' | '\u{4e00}'..='\u{9fa5}' | ':' | '_']+
    rule variable() -> String = n:$(world() (_ world())*) { String::from(n) }
    rule number() -> f64 = n:$("-"? ['0'..='9']+ ("." ['0'..='9']+)?) { n.parse().unwrap() }
    rule boolean() -> bool = "true" { true } / "false" { false }

    // 运算符
    rule calculate() -> Expr = precedence!{
        x:(@) _ "+" _ y:@ { Expr::Plus(Box::new(x), Box::new(y)) }
        x:(@) _ "-" _ y:@ { Expr::Minus(Box::new(x), Box::new(y)) }
        --
        x:(@) _ "*" _ y:@ { Expr::Times(Box::new(x), Box::new(y)) }
        x:(@) _ "/" _ y:@ { Expr::Divide(Box::new(x), Box::new(y)) }
        --
        "(" _ c:calculate() _ ")" { c }
        v:variable() { Expr::NumberVariable(v) }
        n:number() { Expr::Number(Decimal::from_f64(n).unwrap()) }
    }

    // 逻辑运算符
    rule logical() -> Expr = precedence!{
        x:(@) _ "&&" _ y:@ { Expr::And(Box::new(x), Box::new(y)) }
        x:(@) _ "||" _ y:@ { Expr::Or(Box::new(x), Box::new(y)) }
        "!" _ v:@ { Expr::Not(Box::new(v)) }
        --
        x:calculate() _ "==" _ y:calculate() { Expr::Equal(Box::new(x), Box::new(y)) }
        x:calculate() _ "!=" _ y:calculate() { Expr::NotEqual(Box::new(x), Box::new(y)) }
        x:calculate() _ "<"  _ y:calculate() { Expr::LessThan(Box::new(x), Box::new(y)) }
        x:calculate() _ ">"  _ y:calculate() { Expr::GreaterThan(Box::new(x), Box::new(y)) }
        x:calculate() _ "<=" _ y:calculate() { Expr::LessThanEqual(Box::new(x), Box::new(y)) }
        x:calculate() _ ">=" _ y:calculate() { Expr::GreaterThanEqual(Box::new(x), Box::new(y)) }
        --
        b:boolean() { Expr::Boolean(b) }
        v:variable() { Expr::BooleanVariable(v) }
        "(" _ e:logical() _ ")" { e }
    }

    pub(crate) rule parse() -> Expr = _ e:logical() _ { e }
});

/// 解析布尔表达式
///
/// # 参数
///
/// * `input` - 输入字符串
pub(crate) fn parse(input: &str) -> Result<Expr> {
    Ok(bool_parser::parse(input)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let input = "1.0+2.0==3.0";
        let expr = parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::Equal(
                Box::new(Expr::Plus(
                    Box::new(Expr::Number(dec!(1.0))),
                    Box::new(Expr::Number(dec!(2.0)))
                )),
                Box::new(Expr::Number(dec!(3.0)))
            )
        )
    }

    #[test]
    fn test_parse_number_variable() {
        let input = "a+b*c>10";
        let expr = bool_parser::parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::GreaterThan(
                Box::new(Expr::Plus(
                    Box::new(Expr::NumberVariable(String::from("a"))),
                    Box::new(Expr::Times(
                        Box::new(Expr::NumberVariable(String::from("b"))),
                        Box::new(Expr::NumberVariable(String::from("c")))
                    ))
                )),
                Box::new(Expr::Number(dec!(10.0)))
            )
        )
    }

    #[test]
    fn test_parse_boolean_variable() {
        let input = "a&&b||c&&!d";

        let expr = bool_parser::parse(input).unwrap();

        assert_eq!(
            expr,
            Expr::And(
                Box::new(Expr::Or(
                    Box::new(Expr::And(
                        Box::new(Expr::BooleanVariable(String::from("a"))),
                        Box::new(Expr::BooleanVariable(String::from("b")))
                    )),
                    Box::new(Expr::BooleanVariable(String::from("c")))
                )),
                Box::new(Expr::Not(Box::new(Expr::BooleanVariable(String::from(
                    "d"
                )))))
            )
        )
    }
}
