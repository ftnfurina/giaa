use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

//常用结构体

/// 尺寸
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    #[schemars(title = "宽度")]
    pub width: i32,
    #[schemars(title = "高度")]
    pub height: i32,
}

/// 点坐标
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    #[schemars(title = "X 坐标")]
    pub x: i32,
    #[schemars(title = "Y 坐标")]
    pub y: i32,
}

/// 区域
///
/// 左上角坐标为 `start`，右下角坐标为 `end`
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    #[schemars(title = "区域左上角坐标")]
    pub start: Point,
    #[schemars(title = "区域右下角坐标")]
    pub end: Point,
}

/// 为点增加偏移量
///
/// # 参数
///
/// - `point` - 点坐标
/// - `offset_x` - X 轴偏移量
/// - `offset_y` - Y 轴偏移量
pub fn point_offset(point: &Point, offset_x: Option<i32>, offset_y: Option<i32>) -> Point {
    let mut point = *point;
    if let Some(x) = offset_x {
        point.x += x;
    }
    if let Some(y) = offset_y {
        point.y += y;
    }
    point
}

/// 为区域增加偏移量
///
/// # 参数
///
/// - `region` - 区域
/// - `offset_x` - X 轴偏移量
/// - `offset_y` - Y 轴偏移量
pub fn region_offset(region: &Region, offset_x: Option<i32>, offset_y: Option<i32>) -> Region {
    Region {
        start: point_offset(&region.start, offset_x, offset_y),
        end: point_offset(&region.end, offset_x, offset_y),
    }
}

/// 点转为正方形区域
///
/// # 参数
///
/// - `point` - 点坐标
/// - `half_width` - 半宽
pub fn point_to_square_region(point: &Point, half_width: u32) -> Region {
    let start = point_offset(
        point,
        Some(-(half_width as i32)),
        Some(-(half_width as i32)),
    );
    let end = point_offset(point, Some(half_width as i32), Some(half_width as i32));
    Region { start, end }
}

/// 字符串转数字
///
/// # 参数
///
/// - `s` - 字符串
pub fn str_to_number<T: FromStr>(s: &str) -> Result<T> {
    s.chars()
        .filter(|c| c.is_numeric() || c == &'.')
        .collect::<String>()
        .parse::<T>()
        .ok()
        .context("转换数字失败")
}

/// 移除特殊字符
///
/// 包含:
///
/// : ? * | < > " '： ？ ＊ ｜ 《 》 “ ” ‘ ’
///
///
/// # 参数
///
/// - `s` - 字符串
pub fn remove_special_char(s: &str) -> String {
    let chars = vec![
        ':', '?', '*', '|', '<', '>', '"', '\'', '：', '？', '＊', '｜', '《', '》', '“', '”', '‘',
        '’',
    ];
    let mut result = String::new();
    for c in s.chars() {
        if !chars.contains(&c) {
            result.push(c);
        }
    }
    result
}
