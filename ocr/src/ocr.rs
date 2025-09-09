use anyhow::Result;
use image::RgbaImage;

/// OCR 结果
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
}

/// OCR 接口
pub trait Ocr {
    /// 识别图片中的文字
    ///
    /// # 参数
    ///
    /// * `image` - 待识别的图片
    fn recognize(&self, image: &RgbaImage) -> Result<OcrResult>;
}
