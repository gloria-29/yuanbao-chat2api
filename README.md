# yuanbao-chat2api

腾讯元宝Chat2API。

## 免责声明

本项目仅供学习研究用途，请于下载后24小时删除。因本项目造成的风险或损失，开发者不承担任何法律责任。

## 编译

```bash
cargo build --release
```

## 配置

将`config.yml.example`改名为`config.yml`。

打开腾讯元宝官网，F12启动控制台，在网络面板中查看请求，获取必要的字段并填入。

打开终端后执行主程序即可。

## 使用方法

在Cherry Studio里新增一个OpenAI类型的提供者：

![image-20250423105807954](https://public.ptree.top/picgo/2025/04/1745377090.png)

![image-20250423105826414](https://public.ptree.top/picgo/2025/04/1745377107.png)

API密钥随便填。

模型目前只支持DeepSeek R1和V3（注意大小写）。