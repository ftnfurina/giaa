![GIAA](https://socialify.git.ci/ftnfurina/giaa/image?font=Bitter&forks=1&issues=1&language=1&name=1&owner=1&pattern=Floating+Cogs&pulls=1&stargazers=1&theme=Auto)

<div align="center">
  <h1>GIAA (Genshin Impact Artifact Assistant)</h1>
  <p>原神圣遗物助手-圣遗物自动化工具(OCR + 键鼠模拟)</p>
</div>

## 功能介绍

根据圣遗物的名称、双爆得分、星级、套装名、部位等条件, 自动操作圣遗物(锁定、解锁、标记等)

场景举例:

+ 锁定双爆得分超过的30分的圣遗物
+ 锁定携带双爆的圣遗物
+ 锁定双爆得分超过的30分的元素杯子
+ 为所有圣遗物取消锁定和标记
+ 锁定海染套生命值大于1000的部件

## 环境要求

+ 目前只支持 Windows 系统
+ 游戏语言只支持简体中文
+ 游戏窗口比例只支持 16:9

## 使用说明

> [!Warning]
> 工具执行过程中请勿触碰鼠标、键盘、屏幕，否则会导致工具无法正常工作。

> [!Tip]
> 程序可以使用 `--help` 参数查看帮助信息, 使用过程中可以使用鼠标右键来退出程序。

1. 到 [releases](https://github.com/ftnfurina/giaa/releases) 下载最新版本的程序和规则文件
2. 按环境要求检查游戏环境, 推荐使用窗口模式
3. 修改规则文件 [rules.yaml](./rules.yaml), 配置适合自己的规则
4. 打开游戏, 进入圣背包-圣遗物界面
5. 运行程序等待执行完成

**圣遗物名称或套装名无法识别的情况**

1. 请将日志等级设置为 `debug` 并再次尝试, 找到未识别圣遗物的OCR识别结果
```shell
giaa -l debug
```
2. 拷贝圣遗物信息文件 [artifact_info.yaml](./metadata/artifact_info.yaml) 到你程序目录下
```diff
workspace
 ├── giaa.exe
+├── artifact_info.yaml
 └── rules.yaml
```
3. 依据 OCR 识别结果修改 `artifact_info.yaml` 文件, 增加或修改圣遗物别名信息
```diff
# 例如: "烬城勇者绘卷" 无法识别, OCR 识别结果为 "炽城勇者绘卷", 则修改如下:
sets:
  - name: 烬城勇者绘卷
+    alias:
+      - 炽城勇者绘卷
```

## 运行项目

1. 安装 Rust 环境
2. 尝试运行: `cargo run --bin giaa -- --help`
3. 编译项目: `cargo build --release --locked --bin giaa`

## 目录结构

```txt
giaa
 ├── giaa        // 主程序入口
 ├── common      // 公共代码
 ├── metadata    // 圣遗物数据
 ├── model       // OCR 模型转换
 ├── ocr         // 图片文字识别
 ├── parser      // 表达式解析器
 ├── schema      // YAML 配置约束
 ├── window      // 窗口相关代码
 ├── README.md
 ├── Cargo.toml
 └── Cargo.lock
```

## 已知问题

问题较多, 有机会再完善, 突出一个能用就行

+ OCR 识别精度问题: 由 PP-OCRv4_mobile_rec 模型识别，未专门训练过
  + 识别部分生僻字无法识别, 例如: 将帅兜(鍪), (虺)雷之姿
  + 相似字识别有误差, 未做完整测试, 例如: (烬 - 炽)城勇者绘卷
+ 圣遗物添加筛选条件时翻页存在异常, 会提前退出程序

## 参考项目

+ https://github.com/GreatV/oar-ocr
+ https://github.com/wormtql/yas
+ https://github.com/PaddlePaddle/PaddleOCR
+ https://github.com/dvaJi/genshin-data