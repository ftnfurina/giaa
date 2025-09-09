use anyhow::Result;
use common::{Point, Region, Size};
use image::{RgbaImage, imageops};
use tracing::debug;

/// 坐标转换器
pub struct Converter<'a> {
    resolution: &'a Size,
    window_rect: (Point, Size),
}

impl<'a> Converter<'a> {
    /// 创建坐标转换器
    ///
    /// # 参数
    ///
    /// * `resolution` - 适配分辨率
    /// * `window_rect` - 窗口坐标和大小
    pub fn new(resolution: &'a Size, window_rect: (Point, Size)) -> Result<Self> {
        debug!("坐标转换器当前适配分辨率: {:?}", resolution);
        Ok(Self {
            resolution,
            window_rect,
        })
    }

    /// 转换坐标点
    ///
    /// # 参数
    ///
    /// * `point` - 待转换的坐标点
    /// * `with_base` - 是否基于窗口坐标
    pub fn translate_point(&self, point: &Point, with_base: bool) -> Result<Point> {
        let (client, size) = self.window_rect;
        let x = size.width * point.x / self.resolution.width;
        let y = size.height * point.y / self.resolution.height;
        let result = Point {
            x: if with_base { client.x } else { 0 } + x,
            y: if with_base { client.y } else { 0 } + y,
        };
        debug!("坐标转换: {:?} -> {:?}", point, result);
        Ok(result)
    }

    /// 转换区域
    ///
    /// # 参数
    ///
    /// * `region` - 待转换的区域
    pub fn translate_region(&self, region: &Region) -> Result<Region> {
        let start = self.translate_point(&region.start, false)?;
        let end = self.translate_point(&region.end, false)?;
        let result = Region { start, end };
        debug!("区域转换: {:?} -> {:?}", region, result);
        Ok(result)
    }

    /// 裁剪图像
    ///
    /// # 参数
    ///
    /// * `image` - 待裁剪的图像
    /// * `region` - 待裁剪的区域
    pub fn crop_region(&self, image: &RgbaImage, region: &Region) -> Result<RgbaImage> {
        let region = self.translate_region(&region)?;
        debug!("图像裁剪: {:?}", region);
        Ok(imageops::crop_imm(
            image,
            region.start.x as u32,
            region.start.y as u32,
            (region.end.x - region.start.x) as u32,
            (region.end.y - region.start.y) as u32,
        )
        .to_image())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_point() -> Result<()> {
        let resolution = Size {
            width: 1920,
            height: 1080,
        };
        let window_rect = (
            Point { x: 100, y: 50 },
            Size {
                width: 1280,
                height: 720,
            },
        );
        let converter = Converter::new(&resolution, window_rect)?;

        let point = Point { x: 960, y: 540 };
        let translated_point = converter.translate_point(&point, true)?;
        assert_eq!(
            translated_point,
            Point {
                x: 100 + 640,
                y: 50 + 360
            }
        );

        Ok(())
    }

    #[test]
    fn test_translate_region() -> Result<()> {
        let resolution = Size {
            width: 1920,
            height: 1080,
        };
        let window_rect = (
            Point { x: 100, y: 50 },
            Size {
                width: 1280,
                height: 720,
            },
        );
        let converter = Converter::new(&resolution, window_rect)?;

        let region = Region {
            start: Point { x: 0, y: 0 },
            end: Point { x: 1920, y: 1080 },
        };
        let translated_region = converter.translate_region(&region)?;
        assert_eq!(
            translated_region,
            Region {
                start: Point { x: 0, y: 0 },
                end: Point { x: 1280, y: 720 },
            }
        );

        Ok(())
    }

    #[test]
    fn test_crop_region() -> Result<()> {
        let resolution = Size {
            width: 1920,
            height: 1080,
        };
        let window_rect = (
            Point { x: 100, y: 50 },
            Size {
                width: 1280,
                height: 720,
            },
        );
        let converter = Converter::new(&resolution, window_rect)?;

        let image = RgbaImage::new(1920, 1080);
        let region = Region {
            start: Point { x: 0, y: 0 },
            end: Point { x: 1920, y: 1080 },
        };
        let cropped_image = converter.crop_region(&image, &region)?;
        assert_eq!(cropped_image.dimensions(), (1280, 720));

        Ok(())
    }
}
