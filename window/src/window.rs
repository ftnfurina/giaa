use anyhow::Result;
use common::{Point, Size};
use image::RgbaImage;

/// 窗口接口
pub trait Window {
    /// 右键是否按下
    fn is_mouse_right_down(&self) -> bool;
    /// 获取窗口尺寸
    fn rect(&self) -> Result<(Point, Size)>;
    /// 捕获屏幕图像
    fn capture_image(&self) -> Result<RgbaImage>;
    /// 点击窗口坐标点
    ///
    /// # 参数
    ///
    /// * `point` - 点击坐标
    fn click(&self, point: &Point) -> Result<()>;
    /// 垂直滚动窗口
    ///
    /// # 参数
    ///
    /// * `length` - 滚动长度
    fn scroll_vertical(&self, length: i32) -> Result<()>;
    /// 移动鼠标到窗口坐标点
    ///
    /// # 参数
    ///
    /// * `point` - 移动坐标
    fn move_mouse(&self, point: &Point) -> Result<()>;
    /// 尝试获取窗口焦点
    fn try_focus(&self) -> Result<()>;
}
