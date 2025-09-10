use std::{collections::HashMap, fmt};

use metadata::ARTIFACT_INFO;

/// 圣遗物副词条
#[derive(Debug, Clone)]
pub struct ArtifactSubStat {
    pub name: String,
    pub value: f32,
    pub unactivated: bool,
}

/// 圣遗物识别信息
#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub slot: String,
    pub main_stat: String,
    pub main_stat_value: f32,
    pub stars: f32,
    pub sanctifying_elixir: bool,
    pub level: f32,
    pub marked: bool,
    pub locked: bool,
    pub sub_stats: Vec<ArtifactSubStat>,
    pub set_name: String,
    pub equipped: bool,
}

impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "圣遗物 {{ 名称: {}, 部位: {}, 主词条: {}, 主词条值: {}, 星级: {}, 是否祝圣之霜定义: {}, 等级: {}, 是否标记: {}, 是否锁定: {}, 副词条: [{}], 套装名称: {}, 是否装备: {} }}",
            self.name,
            self.slot,
            self.main_stat,
            self.main_stat_value,
            self.stars,
            self.sanctifying_elixir,
            self.level,
            self.marked,
            self.locked,
            self.sub_stats
                .iter()
                .map(|sub_stat| {
                    let unactivated = if sub_stat.unactivated {
                        format!("({})", ARTIFACT_INFO.words.unactivated)
                    } else {
                        String::new()
                    };
                    format!("{}:{}{}", sub_stat.name, sub_stat.value, unactivated)
                })
                .collect::<Vec<String>>()
                .join(", "),
            self.set_name,
            self.equipped
        )
    }
}

impl Artifact {
    /// 获取圣遗物布尔类型的数据
    ///
    /// # 参数
    ///
    /// * `words` - 圣遗物词条信息名称
    pub fn get_boolean_maps(&self) -> HashMap<String, bool> {
        let mut result = HashMap::new();
        result.insert(self.name.clone(), true);
        result.insert(self.slot.clone(), true);
        result.insert(self.set_name.clone(), true);
        result.insert(
            ARTIFACT_INFO.words.sanctifying_elixir.clone(),
            self.sanctifying_elixir,
        );
        result.insert(ARTIFACT_INFO.words.equipped.clone(), self.equipped);
        result.insert(ARTIFACT_INFO.words.marked.clone(), self.marked);
        result.insert(ARTIFACT_INFO.words.locked.clone(), self.locked);
        result
    }

    /// 获取圣遗物数值类型的数据
    ///
    /// # 参数
    ///
    /// * `words` - 圣遗物词条信息名称
    pub fn get_number_maps(&self) -> HashMap<String, f32> {
        let mut result = HashMap::new();
        result.insert(ARTIFACT_INFO.words.star.clone(), self.stars);
        result.insert(ARTIFACT_INFO.words.level.clone(), self.level);
        result.insert(
            format!("{}:{}", ARTIFACT_INFO.words.main_stat, self.main_stat),
            self.main_stat_value,
        );
        for sub_stat in self.sub_stats.iter() {
            result.insert(sub_stat.name.clone(), sub_stat.value);
        }
        result.insert(
            ARTIFACT_INFO.words.sub_stats_count.clone(),
            self.sub_stats
                .iter()
                .filter(|sub_stat| !sub_stat.unactivated)
                .count() as f32,
        );

        result
    }
}

#[derive(Debug)]
// 圣遗物升级材料-祝圣精华/油膏
pub struct ArtifactEnhancementMaterial {
    pub stars: f32,
}

impl fmt::Display for ArtifactEnhancementMaterial {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = if self.stars == 4.0 {
            "祝圣精华"
        } else {
            "祝圣油膏"
        };
        write!(f, "{}", name)
    }
}
