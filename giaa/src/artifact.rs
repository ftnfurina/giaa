use std::{collections::HashMap, fmt};

use metadata::ArtifactWord;

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
    pub sub_stats: Vec<(String, f32)>,
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
                .map(|(name, value)| format!("{}:{}", name, value))
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
    pub fn get_boolean_maps(&self, words: &ArtifactWord) -> HashMap<String, bool> {
        let mut result = HashMap::new();
        result.insert(self.name.clone(), true);
        result.insert(self.slot.clone(), true);
        result.insert(self.set_name.clone(), true);
        result.insert(words.sanctifying_elixir.clone(), self.sanctifying_elixir);
        result.insert(words.equipped.clone(), self.equipped);
        result.insert(words.marked.clone(), self.marked);
        result.insert(words.locked.clone(), self.locked);
        return result;
    }

    /// 获取圣遗物数值类型的数据
    ///
    /// # 参数
    ///
    /// * `words` - 圣遗物词条信息名称
    pub fn get_number_maps(&self, words: &ArtifactWord) -> HashMap<String, f32> {
        let mut result = HashMap::new();
        result.insert(words.star.clone(), self.stars);
        result.insert(words.level.clone(), self.level);
        result.insert(
            format!("{}:{}", words.main_stat, self.main_stat),
            self.main_stat_value,
        );
        for (name, value) in self.sub_stats.iter() {
            result.insert(name.clone(), *value);
        }
        result.insert(words.sub_stats_count.clone(), self.sub_stats.len() as f32);

        return result;
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
