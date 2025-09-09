from paddle2onnx.convert import export

if __name__ == "__main__":
    export(
        model_filename="rec/inference.json",
        params_filename="rec/inference.pdiparams",
        save_file="onnx/PP-OCRv4_mobile_rec_infer.onnx",
    )
