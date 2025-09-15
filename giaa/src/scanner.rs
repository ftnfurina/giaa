use std::{thread, time::Duration};

use crate::{
    actuator::{Actuator, ActuatorResult},
    args::Args,
    color::{average_color_diff, color_distance},
    converter::Converter,
    error::GiaaError,
    identifier::{Identifier, IdentifyResult},
};
use anyhow::{Result, anyhow, bail};
use common::{Point, Region, point_offset, point_to_square_region};
use image::{Pixel, Rgb, RgbaImage};
use metadata::{ARTIFACT_INFO, CoordinateData};
use ocr::{Ocr, OcrResult};
use tracing::{debug, error, info, warn};
use window::Window;

#[derive(Debug)]
struct Scrollbar {
    height: i32,
    button_height: i32,
    scroll_length: i32,
}

/// 扫描器
pub struct Scanner<'a> {
    converter: &'a Converter<'a>,
    coordinate_data: &'a CoordinateData,
    identifier: &'a Identifier<'a>,
    actuator: &'a Actuator<'a>,
    ocr: &'a dyn Ocr,
    window: &'a dyn Window,
    args: &'a Args,
    screenshot: RgbaImage,
    row_index: u32,
    page_scroll_count: u32,
    scroll_count: u32,
    artifact_page_turn_color: image::Rgb<u8>,
    actuator_results: Vec<ActuatorResult>,
}

impl<'a> Scanner<'a> {
    /// 创建扫描器实例
    ///
    /// # 参数
    ///
    /// * `converter` - 坐标转换器
    /// * `coordinate_data` - 坐标数据
    /// * `identifier` - 圣遗物识别器
    /// * `actuator` - 动作执行器
    /// * `ocr` - 文字识别器
    /// * `window` - 窗口接口
    /// * `args` - 程序参数
    pub fn new(
        converter: &'a Converter<'a>,
        coordinate_data: &'a CoordinateData,
        identifier: &'a Identifier<'a>,
        actuator: &'a Actuator<'a>,
        ocr: &'a dyn Ocr,
        window: &'a dyn Window,
        args: &'a Args,
    ) -> Result<Self> {
        Ok(Self {
            converter,
            coordinate_data,
            identifier,
            actuator,
            ocr,
            window,
            args,
            screenshot: RgbaImage::new(0, 0),
            row_index: 0,
            page_scroll_count: 0,
            scroll_count: 0,
            artifact_page_turn_color: image::Rgb([0, 0, 0]),
            actuator_results: vec![],
        })
    }

    /// 刷新截图
    fn refresh_screenshot(&mut self) -> Result<()> {
        self.screenshot = self.window.capture_image()?;
        Ok(())
    }

    /// 识别矩形区域的文字
    ///
    /// # 参数
    ///
    /// * `region` - 待识别的矩形区域
    fn ocr_region(&self, region: &Region) -> Result<OcrResult> {
        self.ocr
            .recognize(&self.converter.crop_region(&self.screenshot, region)?)
    }

    /// 点击坐标
    ///
    /// # 参数
    ///
    /// * `point` - 待点击的坐标
    fn click(&self, point: &Point) -> Result<()> {
        self.window
            .click(&self.converter.translate_point(point, true)?)
    }

    /// 移动鼠标
    ///
    /// # 参数
    ///
    /// * `point` - 待移动的坐标
    fn move_mouse(&self, point: &Point) -> Result<()> {
        self.window
            .move_mouse(&self.converter.translate_point(point, true)?)
    }

    /// 重置圣遗物筛选条件
    fn reset_filter(&self) -> Result<()> {
        info!("开始重置圣遗物筛选条件");
        thread::sleep(Duration::from_millis(500));
        self.click(&self.coordinate_data.artifact_filter_button)?;
        thread::sleep(Duration::from_millis(500));
        self.click(&self.coordinate_data.artifact_filter_reset_button)?;
        thread::sleep(Duration::from_millis(500));
        self.click(&self.coordinate_data.artifact_filter_confirm_button)?;
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    /// 初始化背包状态
    fn init_backpack(&mut self) -> Result<()> {
        info!("开始初始化背包状态");
        let name = self.ocr_region(&self.coordinate_data.backpack_name)?;
        if name.text != ARTIFACT_INFO.words.artifact {
            bail!("未识别到背包圣遗界面");
        }

        if self.args.reset_filter {
            self.reset_filter()?;
        }

        // 检查圣遗物列表是否为空
        self.check_artifact_list_empty_tip()?;

        // 圣遗物详细信息归位
        self.move_mouse(&self.coordinate_data.artifact_detail_center)?;
        self.window
            .scroll_vertical(self.coordinate_data.artifact_detail_scroll_to_top_length)?;
        thread::sleep(Duration::from_secs(1));

        // 圣遗物列表归位
        self.move_mouse(&self.coordinate_data.artifact_list_center)?;
        self.window
            .scroll_vertical(self.coordinate_data.artifact_list_scroll_to_top_length)?;
        thread::sleep(Duration::from_secs(1));

        self.artifact_page_turn_color = self.get_artifact_page_turn()?;
        Ok(())
    }

    /// 检查当前列表是否为空
    fn check_artifact_list_empty_tip(&mut self) -> Result<()> {
        self.refresh_screenshot()?;
        let list_empty_tip = self.ocr_region(&self.coordinate_data.artifact_list_empty_tip)?;
        if list_empty_tip.text == ARTIFACT_INFO.words.no_match_artifacts {
            bail!("未发现圣遗物");
        }
        Ok(())
    }

    /// 获取坐标点的颜色
    ///
    /// # 参数
    ///
    /// * `point` - 坐标点
    fn get_pixel_color(&self, point: &Point) -> Result<Rgb<u8>> {
        let point = self.converter.translate_point(point, false)?;
        Ok(self
            .screenshot
            .get_pixel(point.x as u32, point.y as u32)
            .to_rgb())
    }

    /// 获取滚动条
    fn get_scrollbar(&self) -> Result<Scrollbar> {
        let scrollbar_start = &self.coordinate_data.artifact_list_scrollbar_start;
        let height = self.coordinate_data.artifact_list_scrollbar_height as i32;
        let button_color = Rgb([210, 210, 210]);
        let mut scroll_length = 0;
        let mut button_end_y = 0;
        let mut is_start = false;

        for y in 0..height {
            let point = point_offset(&scrollbar_start, None, Some(y));
            let color = self.get_pixel_color(&point)?;
            let diff = color_distance(&color, &button_color);
            if diff < 10000 {
                if !is_start {
                    scroll_length = y;
                    is_start = true;
                } else {
                    button_end_y = y;
                }
            }
        }

        let button_height = button_end_y - scroll_length;

        Ok(Scrollbar {
            height,
            button_height,
            scroll_length,
        })
    }

    /// 获取圣遗物列表总行数
    fn get_artifact_list_total_rows(&self) -> Result<u32> {
        let scrollbar = self.get_scrollbar()?;
        let total_rows = if self.row_index > self.coordinate_data.artifact_page_rows - 1 {
            // 高行数准确
            (scrollbar.height - scrollbar.button_height) as f32 / scrollbar.scroll_length as f32
                * self.scroll_count as f32
                / (self.scroll_count as f32 / self.row_index as f32)
                + self.coordinate_data.artifact_page_rows as f32
        } else {
            // 低行数准确
            scrollbar.height as f32 / scrollbar.button_height as f32
                * self.coordinate_data.artifact_list_height as f32
                / self.coordinate_data.artifact_list_card_vertical_interval as f32
        };
        Ok(total_rows.floor() as u32)
    }

    /// 检查当前点位是否存在圣遗物卡片
    ///
    /// # 参数
    ///
    /// * `col` - 列数
    /// * `row` - 行数
    ///
    /// # 返回值
    ///
    /// 圣遗物卡片是否存在
    fn check_has_artifact_card(&mut self, col: u32, row: u32) -> Result<bool> {
        let click_point = Point {
            x: self.coordinate_data.artifact_list_card_check_start.x
                + (col * self.coordinate_data.artifact_list_card_horizontal_interval) as i32,
            y: self.coordinate_data.artifact_list_card_check_start.y
                + (row * self.coordinate_data.artifact_list_card_vertical_interval) as i32,
        };

        let regin = point_to_square_region(
            &click_point,
            self.coordinate_data.artifact_list_card_check_width,
        );
        self.refresh_screenshot()?;
        let image = self.converter.crop_region(&self.screenshot, &regin)?;
        let diff = average_color_diff(&image);
        debug!("圣遗物卡片颜色平均差异: {}", diff);
        Ok(diff > 1000)
    }

    /// 扫描当前页的圣遗物
    ///
    /// # 参数
    ///
    /// * `start` - 起始行
    /// * `count` - 识别行数
    /// * `last_count` - 最后一行识别列数
    ///
    /// # 返回值
    ///
    /// 是否是完整的页
    fn scan_now_page(&mut self, start: u32, count: u32) -> Result<bool, GiaaError> {
        info!("识别当前页, 起始行: {}, 识别行数: {} ", start, count);

        for row in start..start + count {
            for col in 0..self.coordinate_data.artifact_page_cols {
                let center = Point {
                    x: self.coordinate_data.artifact_list_card_start.x
                        + (col * self.coordinate_data.artifact_list_card_horizontal_interval)
                            as i32,
                    y: self.coordinate_data.artifact_list_card_start.y
                        + (row * self.coordinate_data.artifact_list_card_vertical_interval) as i32,
                };

                if self.window.is_mouse_right_down() {
                    return Err(GiaaError::RightClickExit);
                }

                self.click(&center)?;
                thread::sleep(Duration::from_millis(self.args.screenshot_delay));
                self.refresh_screenshot()?;

                // 检查是否有圣遗物卡片
                if !self.check_has_artifact_card(col, row)? {
                    return Ok(false);
                }

                match self.identifier.identify(&self.screenshot) {
                    Ok(artifact_result) => match artifact_result {
                        IdentifyResult::Artifact(mut artifact) => {
                            info!("识别到: {}", artifact);
                            let actuator_result = self.actuator.exec(&mut artifact)?;
                            self.actuator_results.push(actuator_result);
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                        IdentifyResult::ArtifactEnhancementMaterial(material) => {
                            info!("识别到: {}", material);
                        }
                    },
                    Err(e) => {
                        error!("识别圣遗物失败: {}", e);
                    }
                }
            }
        }
        Ok(true)
    }

    /// 获取圣遗物翻页颜色
    fn get_artifact_page_turn(&mut self) -> Result<image::Rgb<u8>> {
        self.refresh_screenshot()?;
        self.get_pixel_color(&self.coordinate_data.artifact_page_turn)
    }

    /// 微调行初始位置
    ///
    /// 通过滚轮快速翻页后需要进行微调, 使得下一次识别的行初始位置正确
    fn adjust_row_position(&mut self) -> Result<()> {
        debug!("开始微调行初始位置");
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(150));
            let color = self.get_artifact_page_turn()?;
            let distance = color_distance(&self.artifact_page_turn_color, &color);
            if distance <= 10 {
                return Ok(());
            }
            self.window.scroll_vertical(1)?;
            self.scroll_count += 1;
        }
        Err(anyhow!(
            "调整超出预期, 可能是滚动异常或是圣遗物添加了筛选条件"
        ))
    }

    /// 计算移动所需行数的滚轮次数
    ///
    /// # 参数
    ///
    /// * `row_count` - 行数
    fn calculate_page_scroll_count(&self, row_count: u32) -> u32 {
        self.page_scroll_count / self.coordinate_data.artifact_page_rows * row_count
    }

    /// 移动一行
    fn move_row(&mut self) -> Result<(), GiaaError> {
        let mut changed = false;
        for _ in 0..30 {
            if self.window.is_mouse_right_down() {
                return Err(GiaaError::RightClickExit);
            }
            self.window.scroll_vertical(1)?;
            self.page_scroll_count += 1;
            self.scroll_count += 1;
            thread::sleep(Duration::from_millis(150));

            let color = self.get_artifact_page_turn()?;
            let distance = color_distance(&self.artifact_page_turn_color, &color);
            if changed && distance <= 10 {
                return Ok(());
            } else if !changed && distance > 10 {
                changed = true;
            }
        }
        Err(anyhow!("移动一行失败, 超出最大次数"))?
    }

    /// 移动指定行数
    ///
    /// 首次移动时, 会一行一行的移动, 顺便记录滚动与行数的关系, 下次则直接移动指定行数
    ///
    /// # 参数
    ///
    /// * `row_count` - 行数
    fn move_rows(&mut self, row_count: u32) -> Result<()> {
        info!("开始移动 {} 行", row_count);
        let rows = self.coordinate_data.artifact_page_rows;
        // 移到列表中心
        self.move_mouse(&self.coordinate_data.artifact_list_center)?;

        if self.row_index >= rows {
            for _ in 0..self.calculate_page_scroll_count(row_count) {
                self.window.scroll_vertical(1)?;
                self.scroll_count += 1;
            }
            self.row_index += row_count;
            self.adjust_row_position()?;
        } else {
            for _ in 0..row_count {
                self.move_row()?;
                self.row_index += 1;
            }
        }
        thread::sleep(Duration::from_millis(200));
        Ok(())
    }

    /// 扫描所有页
    fn scan_all_page(&mut self) -> Result<(), GiaaError> {
        let mut page_rows = self.coordinate_data.artifact_page_rows;
        loop {
            let is_full_page = self.scan_now_page(
                self.coordinate_data.artifact_page_rows - page_rows,
                page_rows,
            )?;
            if !is_full_page {
                break;
            }
            self.refresh_screenshot()?;
            let row_count = self.get_artifact_list_total_rows()?;
            let remaining_rows =
                row_count - self.row_index - self.coordinate_data.artifact_page_rows;
            if remaining_rows == 0 {
                break;
            }
            info!("总行数: {}, 剩余行数: {}", row_count, remaining_rows);
            page_rows = self.coordinate_data.artifact_page_rows.min(remaining_rows);
            self.move_rows(page_rows)?;
        }
        Ok(())
    }

    /// 打印处理结果
    fn print_actuator_results(&self) -> Result<()> {
        let mut lock_and_mark_count = 0;
        let mut only_lock_count = 0;
        let mut unlock_and_unmark_count = 0;
        for result in self.actuator_results.iter() {
            match result {
                ActuatorResult::LockAndMark => lock_and_mark_count += 1,
                ActuatorResult::OnlyLock => only_lock_count += 1,
                ActuatorResult::UnlockAndUnmark => unlock_and_unmark_count += 1,
            }
        }
        info!(
            "执行动作结果: 标记(标记和锁定): {}个, 仅锁定: {}个, 未锁定(未标记和未锁定): {}个",
            lock_and_mark_count, only_lock_count, unlock_and_unmark_count
        );
        Ok(())
    }

    /// 开始扫描
    pub fn scan(&mut self) -> Result<()> {
        self.refresh_screenshot()?;
        self.init_backpack()?;
        self.scan_all_page().or_else(|e| match e {
            GiaaError::RightClickExit => {
                warn!("{}", GiaaError::RightClickExit);
                Ok(())
            }
            _ => Err(e),
        })?;
        self.print_actuator_results()
    }
}
