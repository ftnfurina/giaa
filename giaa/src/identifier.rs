use std::{cell::RefCell, collections::HashSet};

use anyhow::{Result, anyhow};
use common::{Point, Region, point_offset, region_offset, remove_special_char, str_to_number};
use image::{Pixel, Rgb, RgbaImage};
use metadata::{ARTIFACT_INFO, CoordinateData};
use ocr::{Ocr, OcrResult};
use parser::Expr;

use crate::{
    args::Args,
    artifact::{Artifact, ArtifactEnhancementMaterial},
    color::{average_color_diff, color_distance},
    converter::Converter,
    rule_expr::RuleExpr,
};

/// 圣遗物识别属性
///
/// 用标明选规则表达式中需要的圣遗物属性
#[derive(Debug, Clone, Copy)]
pub struct ArtifactIdentify {
    pub name: bool,
    pub slot: bool,
    pub main_stat: bool,
    pub main_stat_value: bool,
    pub stars: bool,
    pub sub_stats: bool,
    pub sub_stats_count: bool,
    pub set_name: bool,
    pub equipped: bool,
    pub level: bool,
}

#[derive(Debug)]
pub enum IdentifyResult {
    Artifact(Artifact),                                       // 圣遗物
    ArtifactEnhancementMaterial(ArtifactEnhancementMaterial), // 圣遗物增强材料-祝圣精华/油膏
}

impl ArtifactIdentify {
    /// 创建一个默认的圣遗物识别属性
    pub fn default() -> Self {
        Self {
            name: false,
            slot: false,
            main_stat: false,
            main_stat_value: false,
            stars: false,
            sub_stats: false,
            sub_stats_count: false,
            set_name: false,
            equipped: false,
            level: false,
        }
    }

    /// 通过规则表达式和圣遗物信息, 确定识别哪些字段
    ///
    /// # 参数
    ///
    /// * `rule_exprs` - 规则表达式
    pub fn filter(rule_exprs: &Vec<RuleExpr>) -> Result<Self> {
        let mut all_keys = HashSet::new();
        let mut di = Self::default();

        for rule_expr in rule_exprs {
            let var_keys = Expr::get_var_keys(&rule_expr.expr);
            all_keys.extend(var_keys.boolean_keys);
            all_keys.extend(var_keys.number_keys);
        }

        for name in ARTIFACT_INFO.get_artifact_names() {
            if all_keys.contains(&name) {
                di.name = true;
                break;
            }
        }

        for slot in ARTIFACT_INFO.slots.iter() {
            if all_keys.contains(slot) {
                di.slot = true;
                break;
            }
        }

        if all_keys.contains(&ARTIFACT_INFO.words.star) {
            di.stars = true;
        }

        if all_keys.contains(&ARTIFACT_INFO.words.level) {
            di.level = true;
        }

        for set_name in ARTIFACT_INFO.get_artifact_set_names() {
            if all_keys.contains(&set_name) {
                di.set_name = true;
                break;
            }
        }

        if all_keys.contains(&ARTIFACT_INFO.words.sub_stats_count) {
            di.sub_stats_count = true;
        }

        for stat in ARTIFACT_INFO.stats.iter() {
            if all_keys.contains(&format!("{}:{}", ARTIFACT_INFO.words.main_stat, stat)) {
                di.main_stat = true;
                di.main_stat_value = true;
            }
            if all_keys.contains(stat) {
                di.sub_stats = true;
            }
        }

        // 套装名和副词条个数依赖圣遗物副词条, 所有需要同步开启
        if di.set_name || di.sub_stats_count {
            di.sub_stats = true;
        }

        if all_keys.contains(&ARTIFACT_INFO.words.equipped) {
            di.equipped = true;
        }

        Ok(di)
    }
}

/// 圣遗物识别器
pub struct Identifier<'a> {
    converter: &'a Converter<'a>,
    ocr: &'a dyn Ocr,
    coordinate_data: &'a CoordinateData,
    artifact_identify: &'a ArtifactIdentify,
    args: &'a Args,
    screenshot: RefCell<RgbaImage>,
}

impl<'a> Identifier<'a> {
    /// 创建识别器
    ///
    /// # 参数
    ///
    /// * `converter` - 坐标转换器
    /// * `ocr` - 文字识别器
    /// * `coordinate_data` - 坐标信息
    /// * `artifact_identify` - 识别属性
    pub fn new(
        converter: &'a Converter,
        ocr: &'a dyn Ocr,
        coordinate_data: &'a CoordinateData,
        artifact_identify: &'a ArtifactIdentify,
        args: &'a Args,
    ) -> Result<Self> {
        Ok(Self {
            converter,
            ocr,
            coordinate_data,
            artifact_identify,
            args,
            screenshot: RefCell::new(RgbaImage::new(0, 0)),
        })
    }

    /// 识别截图区域中的文字
    ///
    /// # 参数
    ///
    /// * `region` - 截图区域
    fn ocr_region(&self, region: &Region) -> Result<OcrResult> {
        let image = self
            .converter
            .crop_region(&self.screenshot.borrow(), region)?;
        self.ocr.recognize(&image)
    }

    /// 识别截图区域中的文字, 并偏移y轴
    ///
    /// # 参数
    ///
    /// * `region` - 截图区域
    /// * `offset_y` - 偏移量
    fn ocr_region_offset_y(&self, region: Region, offset_y: i32) -> Result<OcrResult> {
        let region = region_offset(&region, None, Some(offset_y));
        self.ocr_region(&region)
    }

    /// 获取坐标点的颜色
    ///
    /// # 参数
    ///
    /// * `point` - 坐标点
    fn get_pixel_color(&self, point: Point) -> Result<Rgb<u8>> {
        let point = self.converter.translate_point(&point, false)?;
        Ok(self
            .screenshot
            .borrow_mut()
            .get_pixel(point.x as u32, point.y as u32)
            .to_rgb())
    }

    fn is_artifact(&self) -> Result<bool> {
        let region = &self.coordinate_data.artifact_mark_top_right;
        let image = self
            .converter
            .crop_region(&self.screenshot.borrow(), region)?;
        let average_diff = average_color_diff(&image);
        Ok(average_diff > 0)
    }

    /// 识别圣遗物名称
    fn identify_artifact_name(&self) -> Result<String> {
        if self.artifact_identify.name {
            let name = self.ocr_region(&self.coordinate_data.artifact_name)?;
            if let Some(name) = ARTIFACT_INFO.get_artifact_name_by_alias(&name.text) {
                return Ok(name);
            }
            if self.args.strict_mode {
                return Err(anyhow!("未识别到圣遗物名称: {}", name.text));
            }
        }
        Ok(String::new())
    }

    /// 识别圣遗物部位名称
    fn identify_artifact_slot(&self) -> Result<String> {
        if self.artifact_identify.slot {
            let slot = self.ocr_region(&self.coordinate_data.artifact_slot)?;
            if ARTIFACT_INFO.slots.contains(&slot.text) {
                return Ok(slot.text);
            }
            if self.args.strict_mode {
                return Err(anyhow!("未识别到圣遗物部位: {}", slot.text));
            }
        }
        Ok(String::new())
    }

    /// 识别圣遗物主词条名称
    fn identify_artifact_main_stat(&self) -> Result<String> {
        if self.artifact_identify.main_stat {
            let main_stat = self.ocr_region(&self.coordinate_data.artifact_main_stat_name)?;
            if ARTIFACT_INFO.stats.contains(&main_stat.text) {
                return Ok(main_stat.text);
            }
            if self.args.strict_mode {
                return Err(anyhow!("未识别到主属性: {}", main_stat.text));
            }
        }
        Ok(String::new())
    }

    /// 识别圣遗物主词条值
    fn identify_artifact_main_stat_value(&self) -> Result<f32> {
        if self.artifact_identify.main_stat_value {
            let main_stat_value =
                self.ocr_region(&self.coordinate_data.artifact_main_stat_value)?;
            let value = str_to_number::<f32>(&main_stat_value.text);
            if let Ok(value) = value {
                return Ok(value);
            } else if self.args.strict_mode {
                return Err(anyhow!("未识别到主属性值: {}", main_stat_value.text));
            }
        }
        Ok(0.0)
    }

    /// 识别圣遗物星级
    fn identify_artifact_stars(&self) -> Result<f32> {
        if !self.artifact_identify.stars {
            return Ok(0.0);
        }
        let mut star = 2.0;
        let start = self.coordinate_data.artifact_stars_start;
        for i in 2..5 {
            let point = Point {
                x: start.x + i * self.coordinate_data.artifact_stars_horizontal_interval as i32,
                y: start.y,
            };
            let color = self.get_pixel_color(point)?;
            let yellow_color = Rgb([255, 204, 50]);
            let distance = color_distance(&color, &yellow_color);
            if distance > 255 {
                break;
            }
            star += 1.0;
        }
        Ok(star)
    }

    /// 识别圣遗物是否为祝圣之霜定义
    fn identify_artifact_sanctifying_elixir(&self) -> Result<bool> {
        let elixir = self.ocr_region(&self.coordinate_data.artifact_sanctifying_elixir)?;
        Ok(ARTIFACT_INFO.words.sanctifying_elixir == elixir.text)
    }

    /// 识别圣遗物等级
    ///
    /// # 参数
    ///
    /// * `offset` - 偏移量
    fn identify_artifact_level(&self, offset: i32) -> Result<f32> {
        if self.artifact_identify.level {
            let level = self.ocr_region_offset_y(self.coordinate_data.artifact_level, offset)?;
            if let Ok(level) = str_to_number(&level.text) {
                if level < 0.0 || level > 20.0 {
                    return Err(anyhow!("圣遗物等级超出范围: {}", level));
                }

                return Ok(level);
            } else if self.args.strict_mode {
                return Err(anyhow!("未识别到圣遗物等级: {}", level.text));
            }
        }
        Ok(0.0)
    }

    /// 识别圣遗物是否已标记

    fn identify_artifact_marked(&self, offset: i32) -> Result<bool> {
        let point = point_offset(&self.coordinate_data.artifact_mark, None, Some(offset));
        let color = self.get_pixel_color(point)?;

        let white_color = Rgb([255, 255, 255]);
        let distance = color_distance(&color, &white_color);
        Ok(distance > 65025)
    }

    /// 识别圣遗物是否已锁定
    ///
    /// # 参数
    ///
    /// * `offset` - 偏移量
    fn identify_artifact_locked(&self, offset: i32) -> Result<bool> {
        let point = point_offset(&self.coordinate_data.artifact_lock, None, Some(offset));
        let color = self.get_pixel_color(point)?;

        let white_color = Rgb([255, 255, 255]);
        let distance = color_distance(&color, &white_color);
        Ok(distance > 65025)
    }

    /// 识别圣遗物副词条名称和值
    ///
    /// # 参数
    ///
    /// * `offset` - 偏移量
    fn identify_artifact_sub_stats(&self, offset: i32) -> Result<Vec<(String, f32)>> {
        if !self.artifact_identify.sub_stats {
            return Ok(vec![]);
        }
        let mut result = vec![];
        for i in 0..4 {
            let sub_stat_name = self.ocr_region_offset_y(
                self.coordinate_data.artifact_sub_stat_start,
                offset + self.coordinate_data.artifact_sub_stat_height as i32 * i,
            )?;
            let plus_index = sub_stat_name.text.find("+");
            if plus_index.is_none() {
                break;
            }
            let (stat_name, stat_value) = sub_stat_name.text.split_at(plus_index.unwrap());
            let name = stat_name.trim().to_string();

            if !ARTIFACT_INFO.stats.contains(&name) {
                if self.args.strict_mode {
                    return Err(anyhow!("未识别到属性名称: {}", sub_stat_name.text));
                } else {
                    continue;
                }
            }
            let value = str_to_number(&stat_value[1..]);

            if let Ok(value) = value {
                result.push((name, value));
            } else if self.args.strict_mode {
                return Err(anyhow!("未识别到属性值: {}", sub_stat_name.text));
            }
        }
        Ok(result)
    }

    /// 识别圣遗物套装名称
    ///
    /// # 参数
    ///
    /// * `offset` - 偏移量
    fn identify_artifact_set_name(&self, offset: i32) -> Result<String> {
        if self.artifact_identify.set_name {
            let start = Point {
                x: self.coordinate_data.artifact_set_name_x,
                y: self.coordinate_data.artifact_sub_stat_start.start.y,
            };
            let end = Point {
                x: start.x + self.coordinate_data.artifact_set_name_width as i32,
                y: start.y + self.coordinate_data.artifact_set_name_height as i32,
            };
            let set_name = self.ocr_region_offset_y(Region { start, end }, offset)?;
            let set_name = remove_special_char(&set_name.text);
            if let Some(set_name) = ARTIFACT_INFO.get_artifact_set_name_by_alias(&set_name) {
                return Ok(set_name);
            } else if self.args.strict_mode {
                return Err(anyhow!("未识别到套装名称: {}", set_name));
            }
        }
        return Ok(String::new());
    }

    /// 识别圣遗物是否已装备
    fn identify_artifact_equipped(&self) -> Result<bool> {
        if !self.artifact_identify.equipped {
            return Ok(false);
        }
        let equipped = self.ocr_region(&self.coordinate_data.artifact_equipped)?;
        Ok(equipped.text.contains(&ARTIFACT_INFO.words.equipped))
    }

    /// 识别圣遗物信息
    ///
    /// # 参数
    ///
    /// * `screenshot` - 截图
    pub fn identify(&self, screenshot: &RgbaImage) -> Result<IdentifyResult> {
        // todo 添加 OCR 置信度校验
        self.screenshot.replace(screenshot.clone());

        // 检查是否是圣遗物
        let is_artifact = self.is_artifact()?;
        if !is_artifact {
            let stars = self.identify_artifact_stars()?;
            let material = ArtifactEnhancementMaterial { stars };
            return Ok(IdentifyResult::ArtifactEnhancementMaterial(material));
        }

        let mut offset: i32 = 0;

        let name = self.identify_artifact_name()?;
        let slot = self.identify_artifact_slot()?;
        let main_stat = self.identify_artifact_main_stat()?;
        let main_stat_value = self.identify_artifact_main_stat_value()?;
        let stars = self.identify_artifact_stars()?;

        let sanctifying_elixir = self.identify_artifact_sanctifying_elixir()?;
        if sanctifying_elixir {
            offset += self.coordinate_data.artifact_sanctifying_elixir_height as i32;
        }

        let level = self.identify_artifact_level(offset)?;
        let marked = self.identify_artifact_marked(offset)?;
        let locked = self.identify_artifact_locked(offset)?;

        if marked && !locked {
            return Err(anyhow!("圣遗物扫描出已标记未锁定的异常状态"));
        }

        let sub_stats = self.identify_artifact_sub_stats(offset)?;

        offset += self.coordinate_data.artifact_sub_stat_height as i32 * sub_stats.len() as i32;

        let set_name = self.identify_artifact_set_name(offset)?;
        let equipped = self.identify_artifact_equipped()?;

        let artifact = Artifact {
            name,
            slot,
            main_stat,
            main_stat_value,
            stars,
            sub_stats,
            set_name,
            equipped,
            marked,
            locked,
            sanctifying_elixir,
            level,
        };

        Ok(IdentifyResult::Artifact(artifact))
    }
}
