# 模型转换

PaddleOCR (PP-OCRv4_mobile_rec) 模型转为 ONNX 模型

## 使用

1. 安装依赖 `rye sync`
2. 下载 PaddleOCR REC (文字识别) 模型
3. 解压模型文件到 `./rec` 目录下, 目录结构如下:
```
rec
├── inference.json
├── inference.pdiparams
└── inference.yml
```
4. 转换模型 `rye run build-onnx`
5. 转换后的模型位于 `./onnx` 目录下