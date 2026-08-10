# seedream 图像生成接入指南

`doubao-seedream-5-0-260128` 是字节跳动即梦 AI 推出的全新一代生图旗舰模型。具备超强的中文提示词理解力和极其出色的画面细节呈现，是国内目前最顶尖的商业级作画工具。

通过 OpenAI 兼容接口 `POST /v1/images/generations` 调用即可。

---

## 1. 提交生图任务代码示例 (POST)

* **HTTP Method**: `POST`
* **请求路径**: `https://{{domain}}/v1/images/generations`
* **鉴权头部**: `Authorization: Bearer sk-your_token`

### A. 经典文生图（多图并行/顺序生成）

通过 `n` 可一次生成多张图片。

```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "doubao-seedream-5-0-260128",
    "prompt": "一只穿着红色唐装的小柴犬，坐在大门红灯笼下拜年，国潮插画风格，喜庆温馨，4k分辨率",
    "size": "2k",
    "n": 2,
    "watermark": false,
    "response_format": "url"
  }'
```

### B. 图生图与多图参考生图 (Image-to-Image)

支持图生图与多图参考：传入 1 张或多张参考图 URL 即可。

```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "doubao-seedream-5-0-260128",
    "prompt": "参考图一中柴犬的神态和图二的古风房屋背景，绘制一幅精美的贺年国画插图",
    "image_urls": [
      "https://example.com/assets/dog.png",
      "https://example.com/assets/background.png"
    ],
    "size": "2k",
    "n": 1
  }'
```

---

## 2. 完整参数字典说明

| 参数 | 类型 | 必填 | 默认值 | 描述与限制 |
| :--- | :--- | :--- | :--- | :--- |
| `model` | `string` | **是** | - | 图像生成模型，如 `doubao-seedream-5-0-260128`。 |
| `prompt` | `string` | **是** | - | 画面描述提示词。 |
| `size` / `resolution` | `string` | 否 | `"2k"` | 分辨率：像素如 `"1024x720"`，或快捷 `"1k"` / `"2k"` / `"4k"`。 |
| `image` | `string / array` | 否 | - | 参考图 URL（字符串或数组）。 |
| `image_urls` | `array` | 否 | - | 参考图 URL 数组。 |
| `n` | `integer` | 否 | `1` | 生成张数。 |
| `watermark` | `boolean` | 否 | `false` | 是否加水印。 |
| `web_search` | `boolean` | 否 | `false` | 是否启用联网搜索。 |
| `response_format` | `string` | 否 | `"url"` | `"url"` 或 `"b64_json"`。 |

---

## 3. 返回结果示例 (200 OK)

```json
{
  "created": 1719441600,
  "data": [
    {
      "url": "https://example.com/output/img_seedream_dog_1.png"
    },
    {
      "url": "https://example.com/output/img_seedream_dog_2.png"
    }
  ]
}
```
