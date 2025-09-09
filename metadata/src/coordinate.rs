use anyhow::{Result, anyhow};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use common::{Point, Region, Size};

/// 坐标点位数据
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct CoordinateData {
    #[schemars(title = "背包名称识别区域")]
    pub backpack_name: Region,
    #[schemars(title = "圣遗物数量识别区域")]
    pub artifact_count: Region,

    #[schemars(title = "圣遗物每页行数")]
    pub artifact_page_rows: u32,
    #[schemars(title = "圣遗物每页列数")]
    pub artifact_page_cols: u32,

    #[schemars(title = "圣遗物列表卡片起始点(第一行第一列中间偏上)")]
    pub artifact_list_card_start: Point,
    #[schemars(title = "圣遗物列表卡片水平间隔(两圣遗物中心点间隔)")]
    pub artifact_list_card_horizontal_interval: u32,
    #[schemars(title = "圣遗物列表卡片垂直间隔(两圣遗物中心点间隔)")]
    pub artifact_list_card_vertical_interval: u32,

    #[schemars(title = "圣遗物列表卡片检查起始点(第一行第一列右上角)")]
    pub artifact_list_card_check_start: Point,
    #[schemars(title = "圣遗物列表卡片检查区域宽度(判断区域是否为卡片)")]
    pub artifact_list_card_check_width: u32,

    #[schemars(title = "圣遗物详情卡片中心点")]
    pub artifact_detail_center: Point,
    #[schemars(title = "圣遗物详情卡片滚动到顶部长度")]
    pub artifact_detail_scroll_to_top_length: i32,
    #[schemars(title = "圣遗物列表卡片中心点")]
    pub artifact_list_center: Point,
    #[schemars(title = "圣遗物列表滚动到顶部长度")]
    pub artifact_list_scroll_to_top_length: i32,

    #[schemars(title = "圣遗物列表为空提示区域")]
    pub artifact_list_empty_tip: Region,

    #[schemars(title = "圣遗物名称识别区域")]
    pub artifact_name: Region,
    #[schemars(title = "圣遗物部位名称识别区域")]
    pub artifact_slot: Region,
    #[schemars(title = "圣遗物主词条名称识别区域")]
    pub artifact_main_stat_name: Region,
    #[schemars(title = "圣遗物主词条数值识别区域")]
    pub artifact_main_stat_value: Region,

    #[schemars(title = "圣遗物星级起始识别点位")]
    pub artifact_stars_start: Point,
    #[schemars(title = "圣遗物星级星星水平间隔")]
    pub artifact_stars_horizontal_interval: u32,

    #[schemars(title = "圣遗物祝圣之霜定义识别区域")]
    pub artifact_sanctifying_elixir: Region,
    #[schemars(title = "圣遗物祝圣之霜定义高度")]
    pub artifact_sanctifying_elixir_height: u32,

    #[schemars(title = "圣遗物等级识别区域")]
    pub artifact_level: Region,

    #[schemars(title = "圣遗物锁定识别点位")]
    pub artifact_lock: Point,
    #[schemars(title = "圣遗物标记识别点位")]
    pub artifact_mark: Point,

    #[schemars(title = "圣遗物副词条起始识别区域")]
    pub artifact_sub_stat_start: Region,
    #[schemars(title = "圣遗物副词条高度")]
    pub artifact_sub_stat_height: u32,

    // 套装名称依据副词条动态调整识别区域(套装名和副词条的文本高度相同)
    #[schemars(title = "圣遗物套装名称识别横坐标")]
    pub artifact_set_name_x: i32,
    #[schemars(title = "圣遗物套装名称识别宽度")]
    pub artifact_set_name_width: u32,
    #[schemars(title = "圣遗物套装名称识别高度")]
    pub artifact_set_name_height: u32,

    #[schemars(title = "圣遗物是否装备识别区域")]
    pub artifact_equipped: Region,

    // 默认点位是第一行第一列卡片顶部与列表边框中间
    // 点位必须满足要求:
    // 1. 圣遗物卡片对齐顶部时, 点位的颜色能够取到背景色
    // 2. 圣遗物卡片未对齐顶部时, 点位的颜色能够取到圣遗物卡片的颜色
    #[schemars(title = "圣遗物行是否对齐顶部识别点位")]
    pub artifact_page_turn: Point,

    #[schemars(title = "圣遗物筛选按钮点位")]
    pub artifact_filter_button: Point,
    #[schemars(title = "圣遗物筛选重置按钮点位")]
    pub artifact_filter_reset_button: Point,
    #[schemars(title = "圣遗物筛选确认按钮点位")]
    pub artifact_filter_confirm_button: Point,
}

/// 坐标数据
#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct Coordinate {
    #[schemars(title = "适配分辨率")]
    pub resolution: Size,
    #[schemars(title = "描述")]
    pub description: Option<String>,
    #[schemars(title = "数据")]
    pub data: CoordinateData,
}

impl Coordinate {
    /// 加载所有分辨率的坐标数据
    pub fn load_all() -> Result<Vec<Self>> {
        Ok(vec![serde_yaml::from_str(include_str!(
            "../coordinates/1920x1080.yaml"
        ))?])
    }

    /// 加载指定分辨率的坐标数据
    ///
    /// # 参数
    ///
    /// * `resolution` - 适配分辨率
    pub fn load(resolution: Size) -> Result<Self> {
        for coord in Self::load_all()? {
            if resolution.width * coord.resolution.height
                == resolution.height * coord.resolution.width
            {
                return Ok(coord);
            }
        }
        Err(anyhow!("没有找到适配分辨率为 {:?} 的坐标数据", resolution))
    }
}
