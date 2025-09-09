use std::cell::RefCell;

use anyhow::{Result, anyhow};
use image::{
    RgbaImage,
    imageops::{self, FilterType},
};

use ndarray::{Array, ArrayBase, Dim, OwnedRepr};
use ort::{
    session::{Session, SessionOutputs, builder::GraphOptimizationLevel},
    value::TensorRef,
};
use tracing::debug;

use crate::ocr::{Ocr, OcrResult};

/// 基于 PaddleOCR 的 OCR 实现
pub struct PPOcr {
    session: RefCell<Session>,
    character_dict: Vec<String>,
}

impl PPOcr {
    /// 创建 PPOcr 实例
    pub fn new() -> Result<PPOcr> {
        let model_bytes = include_bytes!("../PP-OCRv4_mobile_rec_infer.onnx");
        let character_dict = include_str!("../character_dict.txt")
            .lines()
            .map(String::from)
            .collect();
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_memory(model_bytes)?;

        debug!("PP-OCRv4 OCR 模型加载成功");

        Ok(PPOcr {
            session: RefCell::new(session),
            character_dict,
        })
    }

    /// 将图像转换为张量数组数据
    ///
    /// # 参数
    ///
    /// * `image` - 输入图像
    fn image_to_tensor_array_data(image: &RgbaImage) -> ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>> {
        let (width, height) = image.dimensions();

        let target_height = 48;
        let target_width = ((width as f32 / height as f32) * target_height as f32) as u32;

        let resized_image =
            imageops::resize(image, target_width, target_height, FilterType::Triangle);
        let mut input = Array::zeros((1, 3, target_height as usize, target_width as usize));

        for (x, y, pixel) in resized_image.enumerate_pixels() {
            let [r, g, b, _] = pixel.0;

            input[[0, 0, y as usize, x as usize]] = r as f32 / 255.0;
            input[[0, 1, y as usize, x as usize]] = g as f32 / 255.0;
            input[[0, 2, y as usize, x as usize]] = b as f32 / 255.0;
        }
        input
    }

    /// 处理模型输出
    ///
    /// # 参数
    ///
    /// * `outputs` - 模型输出
    fn handle_session_outputs(&self, outputs: &SessionOutputs) -> Result<OcrResult> {
        let (output_shape, output_data) = outputs[0].try_extract_tensor::<f32>()?;

        if output_shape.len() != 3 {
            return Err(anyhow!("意想不到的输出形状: {:?}", output_shape));
        }

        let batch_size_out = output_shape[0] as usize;
        let seq_len = output_shape[1] as usize;
        let num_classes = output_shape[2] as usize;
        let expected_len = batch_size_out * seq_len * num_classes;

        if output_data.len() != expected_len {
            return Err(anyhow!("意想不到的输出长度: {}", output_data.len()));
        }

        let array_view =
            ndarray::ArrayView3::from_shape((batch_size_out, seq_len, num_classes), output_data)
                .map_err(|e| anyhow!("转换输出到数组视图失败: {}", e))?;

        let pred = array_view.to_owned();
        let blank_index = 0;

        let preds = pred.index_axis(ndarray::Axis(0), 0);

        let mut sequence_idx = Vec::new();
        let mut sequence_prob = Vec::new();

        for row in preds.outer_iter() {
            if let Some((idx, &prob)) = row
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            {
                sequence_idx.push(idx);
                sequence_prob.push(prob);
            }
        }

        let mut filtered_idx = Vec::new();
        let mut filtered_prob = Vec::new();

        for (i, &idx) in sequence_idx.iter().enumerate() {
            if (i > 0 && sequence_idx[i] == sequence_idx[i - 1]) || idx == blank_index {
                continue;
            }

            filtered_idx.push(idx);
            filtered_prob.push(sequence_prob[i]);
        }

        if filtered_idx.is_empty() {
            return Ok(OcrResult {
                text: "".to_string(),
                confidence: 0.0,
            });
        }

        let text: String = filtered_idx
            .iter()
            .map(|&idx| self.character_dict[idx - 1].clone())
            .collect::<String>()
            .trim()
            .to_string();

        let confidence = filtered_prob.iter().sum::<f32>() / filtered_prob.len() as f32;

        debug!("识别结果: {}, 置信度: {}", text, confidence);

        Ok(OcrResult { text, confidence })
    }
}

impl Ocr for PPOcr {
    /// 识别图像中的文本
    ///
    /// # 参数
    ///
    /// * `image` - 输入图像
    fn recognize(&self, image: &RgbaImage) -> Result<OcrResult> {
        let tensor = PPOcr::image_to_tensor_array_data(image);
        let tensor = TensorRef::from_array_view(tensor.view())?;
        let mut session = self.session.borrow_mut();
        let outputs = session.run(ort::inputs![tensor])?;
        self.handle_session_outputs(&outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognize() {
        let pp_ocr = PPOcr::new().unwrap();
        let image = image::open("test.png").unwrap().to_rgba8();
        let result = pp_ocr.recognize(&image).unwrap();
        assert_eq!(result.text, "宗室面具");
    }
}
