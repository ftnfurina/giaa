use anyhow::{Result, anyhow, bail};
use common::{Point, Size};
use image::RgbaImage;
use std::{cell::RefCell, mem, thread, time::Duration};

use enigo::Axis;
use enigo::Button;
use enigo::{Coordinate, Direction, Enigo, Mouse, Settings};
use tracing::debug;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_RBUTTON},
        WindowsAndMessaging::{
            GetWindowInfo, SW_RESTORE, SetForegroundWindow, ShowWindow, WINDOWINFO,
        },
    },
};
use xcap::Window as WindowXCap;

use crate::window::Window;

/// 设置进程 DPI 缩放感知
fn set_dpi_awareness() -> Result<()> {
    use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};
    let _ = unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) }
        .map_err(|_| anyhow!("设置进程 DPI 缩放感知失败"));
    debug!("设置进程 DPI 缩放感知完成");
    Ok(())
}

/// 寻找游戏窗口
///
/// # 参数
///
/// * `titles` - 窗口标题列表
fn find_window(titles: &Vec<String>) -> Result<WindowXCap> {
    for window in WindowXCap::all()? {
        if titles.contains(&window.title()?) {
            debug!("成功找到名为 '{}' 的窗口", window.title()?);
            return Ok(window);
        }
    }
    Err(anyhow!(
        "未找到游戏窗口, 请确认游戏窗口名称与输入的参数是否匹配"
    ))
}

/// Windows 平台窗口实现
pub struct WinWindow {
    window: RefCell<WindowXCap>,
    enigo: RefCell<Enigo>,
}

impl WinWindow {
    /// 创建窗口实例
    ///
    /// # 参数
    ///
    /// * `titles` - 窗口标题列表
    pub fn new(titles: &Vec<String>) -> Result<Self> {
        set_dpi_awareness()?;
        Ok(Self {
            window: RefCell::new(find_window(titles)?),
            enigo: RefCell::new(Enigo::new(&Settings::default())?),
        })
    }

    /// 获取窗口句柄
    fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.borrow().id()? as _))
    }

    /// 获取窗口信息
    fn window_info(&self) -> Result<WINDOWINFO> {
        let hwnd = self.hwnd()?;

        let mut window_info = WINDOWINFO {
            cbSize: mem::size_of::<WINDOWINFO>() as u32,
            ..WINDOWINFO::default()
        };

        unsafe {
            GetWindowInfo(hwnd, &mut window_info)?;
        };

        Ok(window_info)
    }

    /// 窗口置顶
    fn focus(&self) -> Result<()> {
        let hwnd = self.hwnd()?;
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    /// 滑动滚轮
    ///
    /// # 参数
    ///
    /// * `length` - 滑动长度
    /// * `axis` - 滑动方向
    fn scroll(&self, length: i32, axis: Axis) -> Result<()> {
        let mut enigo = self.enigo.borrow_mut();
        debug!("滚动 {} 步", length);
        for _ in 0..length.abs() {
            enigo.scroll(if length > 0 { 1 } else { -1 }, axis)?;
        }
        Ok(())
    }

    /// 显示当前所有的窗口名称
    pub fn list_window_titles() -> Result<Vec<String>> {
        let mut titles = vec![];
        for window in WindowXCap::all()? {
            let title = window.title()?;
            if title.is_empty() {
                continue;
            }
            titles.push(title);
        }
        Ok(titles)
    }
}

impl Window for WinWindow {
    /// 右键是否按下
    fn is_mouse_right_down(&self) -> bool {
        unsafe {
            let state = GetAsyncKeyState(VK_RBUTTON.0 as i32);
            if state == 0 {
                return false;
            }
            state & 1 > 0
        }
    }

    /// 获取窗口尺寸
    fn rect(&self) -> Result<(Point, Size)> {
        let info = self.window_info()?;
        let rc_client = info.rcClient;
        let width = rc_client.right - rc_client.left;
        let height = rc_client.bottom - rc_client.top;
        Ok((
            Point {
                x: rc_client.left,
                y: rc_client.top,
            },
            Size { width, height },
        ))
    }

    /// 捕获屏幕图像
    fn capture_image(&self) -> Result<RgbaImage> {
        Ok(self.window.borrow().capture_image()?)
    }

    /// 点击窗口坐标点
    ///
    /// # 参数
    ///
    /// * `point` - 点击坐标
    fn click(&self, point: &Point) -> Result<()> {
        debug!("点击坐标: ({}, {})", point.x, point.y);
        let mut enigo = self.enigo.borrow_mut();
        enigo.move_mouse(point.x, point.y, Coordinate::Abs)?;
        Ok(enigo.button(Button::Left, Direction::Click)?)
    }

    /// 垂直滚动窗口
    ///
    /// # 参数
    ///
    /// * `length` - 滚动长度
    fn scroll_vertical(&self, length: i32) -> Result<()> {
        debug!("垂直滚动 {} 步", length);
        self.scroll(length, Axis::Vertical)
    }

    /// 移动鼠标到窗口坐标点
    ///
    /// # 参数
    ///
    /// * `point` - 移动坐标
    fn move_mouse(&self, point: &Point) -> Result<()> {
        debug!("移动到坐标: ({}, {})", point.x, point.y);
        Ok(self
            .enigo
            .borrow_mut()
            .move_mouse(point.x, point.y, Coordinate::Abs)?)
    }

    /// 尝试获取窗口焦点
    fn try_focus(&self) -> Result<()> {
        self.focus()?;
        thread::sleep(Duration::from_secs(1));

        if !self.window.borrow().is_focused()? {
            bail!("窗口失去焦点, 无法进行后续操作");
        }
        Ok(())
    }
}
