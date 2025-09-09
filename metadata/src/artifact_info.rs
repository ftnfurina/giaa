use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 圣遗物名称
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    #[schemars(title = "圣遗物名称")]
    pub name: String,
    #[schemars(title = "圣遗物别名(可用于 OCR 识别异常补救)")]
    pub alias: Option<Vec<String>>,
}

/// 圣遗物套装
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactSet {
    #[schemars(title = "套装名称")]
    pub name: String,
    #[schemars(title = "套装别名(可用于 OCR 识别异常补救)")]
    pub alias: Option<Vec<String>>,
    #[schemars(title = "圣遗物")]
    pub artifacts: Vec<Artifact>,
}

/// 圣遗物相关词汇
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactWord {
    #[schemars(title = "圣遗物")]
    pub artifact: String,
    #[schemars(title = "星级")]
    pub star: String,
    #[schemars(title = "等级")]
    pub level: String,
    #[schemars(title = "已装备")]
    pub equipped: String,
    #[schemars(title = "已锁定")]
    pub locked: String,
    #[schemars(title = "已标记")]
    pub marked: String,
    #[schemars(title = "祝圣之霜定义")]
    pub sanctifying_elixir: String,
    #[schemars(title = "主词条前缀")]
    pub main_stat: String,
    #[schemars(title = "副词条个数")]
    pub sub_stats_count: String,

    #[schemars(title = "暂无满足条件的圣遗物")]
    pub no_match_artifacts: String,
}

/// 圣遗物信息
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactInfo {
    #[schemars(title = "圣遗物部位名称")]
    pub slots: Vec<String>,
    #[schemars(title = "圣遗物属性名称")]
    pub stats: Vec<String>,
    #[schemars(title = "圣遗物套装")]
    pub sets: Vec<ArtifactSet>,
    #[schemars(title = "圣遗物词汇")]
    pub words: ArtifactWord,
    #[serde(skip)]
    artifact_name_map_cache: RefCell<HashMap<String, String>>,
    #[serde(skip)]
    artifact_set_names_cache: RefCell<HashMap<String, String>>,
}

impl ArtifactInfo {
    /// 加载解析圣遗物信息文件
    pub fn new() -> Result<Self> {
        Ok(serde_yaml::from_str(include_str!("../artifact_info.yaml"))?)
    }

    /// 获取所有圣遗物名称映射
    fn get_artifact_name_map(&self) -> HashMap<String, String> {
        let mut map_cache = self.artifact_name_map_cache.borrow_mut();
        if map_cache.is_empty() {
            for set in self.sets.iter() {
                for artifact in set.artifacts.iter() {
                    map_cache.insert(artifact.name.clone(), artifact.name.clone());
                    if let Some(alias) = &artifact.alias {
                        for name in alias.iter() {
                            map_cache.insert(name.clone(), artifact.name.clone());
                        }
                    }
                }
            }
        }
        map_cache.clone()
    }

    /// 获取所有圣遗物套装名称映射
    fn get_artifact_set_name_map(&self) -> HashMap<String, String> {
        let mut map_cache = self.artifact_set_names_cache.borrow_mut();
        if map_cache.is_empty() {
            for set in self.sets.iter() {
                map_cache.insert(set.name.clone(), set.name.clone());
                if let Some(alias) = &set.alias {
                    for name in alias.iter() {
                        map_cache.insert(name.clone(), set.name.clone());
                    }
                }
            }
        }
        map_cache.clone()
    }

    /// 获取所有圣遗物名称, 不包含别名
    pub fn get_artifact_names(&self) -> Vec<String> {
        self.get_artifact_name_map().values().cloned().collect()
    }

    /// 获取所有圣遗物套装名称, 不包含别名
    pub fn get_artifact_set_names(&self) -> Vec<String> {
        self.get_artifact_set_name_map().values().cloned().collect()
    }

    /// 通过别名获取圣遗物名称
    ///
    /// # 参数
    ///
    /// * `alias` - 圣遗物别名
    pub fn get_artifact_name_by_alias(&self, alias: &str) -> Option<String> {
        self.get_artifact_name_map().get(alias).cloned()
    }

    /// 通过别名获取圣遗物套装名称
    ///
    /// # 参数
    ///
    /// * `alias` - 圣遗物套装别名
    pub fn get_artifact_set_name_by_alias(&self, alias: &str) -> Option<String> {
        self.get_artifact_set_name_map().get(alias).cloned()
    }

    /// 获取所有布尔型关键字
    pub fn get_boolean_keys(&self) -> Vec<String> {
        let mut result = vec![
            self.words.equipped.clone(),
            self.words.marked.clone(),
            self.words.locked.clone(),
            self.words.sanctifying_elixir.clone(),
        ];

        result.extend(self.slots.clone());
        result.extend(self.stats.clone());
        result.extend(self.get_artifact_set_names());
        result.extend(self.get_artifact_names());
        result
    }

    /// 获取所有数字型关键字
    pub fn get_number_keys(&self) -> Vec<String> {
        let mut result = vec![
            self.words.star.clone(),
            self.words.level.clone(),
            self.words.sub_stats_count.clone(),
        ];

        for stat in self.stats.iter() {
            result.push(stat.clone());
            result.push(format!("{}:{}", self.words.main_stat, stat));
        }
        result
    }
}
