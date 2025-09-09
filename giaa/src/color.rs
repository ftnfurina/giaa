use image::Pixel;
use image::RgbaImage;

/// 颜色距离
///
/// # 参数
///
/// * `c1` - 颜色1
/// * `c2` - 颜色2
pub fn color_distance(c1: &image::Rgb<u8>, c2: &image::Rgb<u8>) -> i32 {
    let x = c1.0[0] as i32 - c2.0[0] as i32;
    let y = c1.0[1] as i32 - c2.0[1] as i32;
    let z = c1.0[2] as i32 - c2.0[2] as i32;
    (x * x + y * y + z * z).abs()
}

/// 图片平均颜色差异
///
/// 计算相邻像素的颜色差异, 并求和除以总数, 得到平均颜色差异.
///
/// # 参数
///
/// * `image` - 待比较的图片
pub fn average_color_diff(image: &RgbaImage) -> i32 {
    let (width, height) = image.dimensions();
    let mut count: f32 = 0.0;
    let mut diffs: Vec<i32> = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if x + 1 < width {
                let c1 = image.get_pixel(x, y).to_rgb();
                let c2 = image.get_pixel(x + 1, y).to_rgb();
                diffs.push(color_distance(&c1, &c2));
                count += 1.0;
            }

            if y + 1 < height {
                let c1 = image.get_pixel(x, y).to_rgb();
                let c2 = image.get_pixel(x, y + 1).to_rgb();
                diffs.push(color_distance(&c1, &c2));
                count += 1.0;
            }
        }
    }
    diffs
        .iter()
        .map(|diff| *diff as f32 / count as f32)
        .sum::<f32>() as i32
}

#[cfg(test)]
mod tests {
    use image::{Rgb, Rgba};

    use super::*;

    #[test]
    fn test_average_color_diff_happy_path() {
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        img.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        img.put_pixel(1, 1, Rgba([255, 255, 0, 255]));

        let expected_diff = color_distance(&Rgb([255, 0, 0]), &Rgb([0, 255, 0])) as f32
            + color_distance(&Rgb([255, 0, 0]), &Rgb([0, 0, 255])) as f32
            + color_distance(&Rgb([0, 255, 0]), &Rgb([255, 255, 0])) as f32
            + color_distance(&Rgb([0, 0, 255]), &Rgb([255, 255, 0])) as f32;

        let avg_diff = average_color_diff(&img);
        assert_eq!(avg_diff, (expected_diff / 4.0) as i32);
    }

    #[test]
    fn test_average_color_diff_single_pixel() {
        let img = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));

        let avg_diff = average_color_diff(&img);
        assert_eq!(avg_diff, 0);
    }

    #[test]
    fn test_average_color_diff_uniform_image() {
        let img = RgbaImage::from_pixel(10, 10, Rgba([128, 128, 128, 255]));

        let avg_diff = average_color_diff(&img);
        assert_eq!(avg_diff, 0);
    }

    #[test]
    fn test_average_color_diff_edge_case() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255]));

        let expected_diff = color_distance(&Rgb([255, 0, 0]), &Rgb([0, 255, 0])) as f32;

        let avg_diff = average_color_diff(&img);
        assert_eq!(avg_diff, (expected_diff / 1.0) as i32);
    }

    #[test]
    fn test_average_color_diff_vertical_edge_case() {
        let mut img = RgbaImage::new(1, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(0, 1, Rgba([0, 255, 0, 255]));

        let expected_diff = color_distance(&Rgb([255, 0, 0]), &Rgb([0, 255, 0])) as f32;

        let avg_diff = average_color_diff(&img);
        assert_eq!(avg_diff, (expected_diff / 1.0) as i32);
    }
}
