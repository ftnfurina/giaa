use std::collections::HashMap;

use anyhow::Result;
use common::{Point, point_offset};
use metadata::{ARTIFACT_INFO, Coordinate, RuleAction};
use parser::{ExprResult, ExprVar, ExprVarKey, Parser};
use tracing::info;
use window::Window;

use crate::{artifact::Artifact, converter::Converter, rule_expr::RuleExpr};

#[derive(Debug)]
pub enum ActuatorResult {
    UnlockAndUnmark,
    OnlyLock,
    LockAndMark,
}

/// 动作执行器, 依据规则表达式和圣遗物识别信息, 执行动作
pub struct Actuator<'a> {
    parser: &'a Parser,
    coordinate: &'a Coordinate,
    converter: &'a Converter<'a>,
    window: &'a dyn Window,
    rule_exprs: &'a Vec<RuleExpr>,
}

impl<'a> Actuator<'a> {
    /// 构造动作执行器
    ///
    /// # 参数
    ///
    /// * `parser` - 表达式解析器
    /// * `window` - 窗口接口
    /// * `converter` - 坐标转换器
    /// * `rule_exprs` - 规则与表达式映射列表
    /// * `coordinate` - 坐标数据
    pub fn new(
        parser: &'a Parser,
        window: &'a dyn Window,
        converter: &'a Converter,
        rule_exprs: &'a Vec<RuleExpr>,
        coordinate: &'a Coordinate,
    ) -> Result<Self> {
        Ok(Self {
            parser: parser,
            window,
            converter,
            rule_exprs,
            coordinate,
        })
    }

    /// 点击坐标
    ///
    /// # 参数
    ///
    /// * `point` - 点击坐标
    fn click(&self, point: &Point) -> Result<()> {
        self.window
            .click(&self.converter.translate_point(point, true)?)?;
        Ok(())
    }

    /// 识别为 "祝圣之霜定义" 时添加高度偏移
    ///
    /// # 参数
    ///
    /// * `point` - 点击坐标
    /// * `artifact` - 圣遗物识别信息
    fn sanctifying_elixir_offset(&self, point: &Point, artifact: &Artifact) -> Point {
        if artifact.sanctifying_elixir {
            return point_offset(
                &point,
                None,
                Some(self.coordinate.data.artifact_sanctifying_elixir_height as i32),
            );
        }
        point.clone()
    }

    /// 点击锁定按钮
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物识别信息
    fn click_lock(&self, artifact: &Artifact) -> Result<()> {
        self.click(&self.sanctifying_elixir_offset(&self.coordinate.data.artifact_lock, artifact))
    }

    /// 点击标记按钮
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物识别信息
    fn click_mark(&self, artifact: &Artifact) -> Result<()> {
        self.click(&self.sanctifying_elixir_offset(&self.coordinate.data.artifact_mark, artifact))
    }

    /// 处理锁定和标记按钮
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物识别信息
    fn handle_only_lock(&self, artifact: &Artifact) -> Result<()> {
        if artifact.locked {
            if artifact.marked {
                return self.click_mark(artifact);
            }
        } else {
            return self.click_lock(artifact);
        }
        Ok(())
    }

    /// 处理锁定和标记按钮
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物识别信息
    fn handle_lock_and_mark(&self, artifact: &Artifact) -> Result<()> {
        if !artifact.marked {
            return self.click_mark(artifact);
        }
        Ok(())
    }

    /// 处理解锁锁定和标记按钮
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物识别信息
    fn handle_un_lock_and_mark(&self, artifact: &Artifact) -> Result<()> {
        if artifact.locked {
            return self.click_lock(artifact);
        }
        Ok(())
    }

    /// 将圣遗物信息转换为表达式变量
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物
    /// * `expr_var_key` - 表达式变量键
    fn generate_vars(&self, artifact: &Artifact, expr_var_key: &ExprVarKey) -> ExprVar {
        // 布尔变量
        let mut boolean_vars = HashMap::new();
        for name in ARTIFACT_INFO.get_artifact_names() {
            boolean_vars.insert(name, false);
        }
        for slot in ARTIFACT_INFO.slots.iter() {
            boolean_vars.insert(slot.clone(), false);
        }
        for set_name in ARTIFACT_INFO.get_artifact_set_names() {
            boolean_vars.insert(set_name, false);
        }
        boolean_vars.extend(artifact.get_boolean_maps());

        // 数字变量
        let mut number_vars = HashMap::new();
        for stat in ARTIFACT_INFO.stats.iter() {
            number_vars.insert(stat.clone(), 0.0);
            number_vars.insert(format!("{}:{}", ARTIFACT_INFO.words.main_stat, stat), 0.0);
        }
        number_vars.extend(artifact.get_number_maps());

        // 筛选表达式所需要的变量
        let boolean_vars = boolean_vars
            .iter()
            .filter_map(|(name, value)| {
                if expr_var_key.boolean_keys.contains(&name) {
                    Some((name.clone(), *value))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        let number_vars = number_vars
            .iter()
            .filter_map(|(name, value)| {
                if expr_var_key.number_keys.contains(&name) {
                    Some((name.clone(), *value))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        ExprVar {
            boolean_vars,
            number_vars,
        }
    }

    /// 执行动作, 并返回更新后的圣遗物信息
    ///
    /// # 参数
    ///
    /// * `artifact` - 圣遗物
    pub fn exec(&self, artifact: &mut Artifact) -> Result<ActuatorResult> {
        // 保留圣遗物原始状态
        let before_artifact = artifact.clone();

        // 先计算圣遗物交换状态
        for rule_expr in self.rule_exprs.iter() {
            let expr_var = self.generate_vars(&artifact, &rule_expr.expr_var_key);

            if let ExprResult::Boolean(result) = self.parser.exec(&rule_expr.expr, &expr_var)? {
                if !result {
                    continue;
                }
                info!("规则命中: {}", rule_expr.rule.description);
                match rule_expr.rule.action {
                    RuleAction::ClickLock => {
                        artifact.locked = !artifact.locked;
                        if artifact.marked && !artifact.locked {
                            artifact.marked = false;
                        }
                    }
                    RuleAction::ClickMark => {
                        artifact.marked = !artifact.marked;
                        if !artifact.locked && artifact.marked {
                            artifact.locked = true;
                        }
                    }
                    RuleAction::Lock => {
                        artifact.locked = true;
                    }
                    RuleAction::OnlyLock => {
                        artifact.locked = true;
                        artifact.marked = false;
                    }
                    RuleAction::LockAndMark => {
                        artifact.locked = true;
                        artifact.marked = true;
                    }
                    RuleAction::UnLockAndMark => {
                        artifact.locked = false;
                        artifact.marked = false;
                    }
                }
            }
        }

        // 更改状态
        let result = if artifact.locked {
            if artifact.marked {
                self.handle_lock_and_mark(&before_artifact)?;
                ActuatorResult::LockAndMark
            } else {
                self.handle_only_lock(&before_artifact)?;
                ActuatorResult::OnlyLock
            }
        } else {
            if artifact.marked {
                unreachable!("不存在未锁定但标记的圣遗物")
            } else {
                self.handle_un_lock_and_mark(&before_artifact)?;
                ActuatorResult::UnlockAndUnmark
            }
        };
        Ok(result)
    }
}
