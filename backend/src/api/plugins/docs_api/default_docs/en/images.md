# Image Generation & Editing Endpoints

OpenAI-compatible image APIs. Use the endpoints and parameters below.

### 1. Image Generations
* **Path**: `/v1/images/generations`
* **Method**: `POST`

#### Major Request Parameters
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `model` | `string` | Yes | Image model name, e.g. `dall-e-3`, `wanx-v1`, `seedream-5.0-lite` |
| `prompt` | `string` | Yes | Text description of the desired image(s) |
| `n` | `integer` | No | Number of images (default `1`) |
| `size` | `string` | No | Resolution, e.g. `1024x1024` |
| `resolution` | `string` | No | Alternate resolution field (e.g. `1k` / `2k`); use with models that expect it |
| `response_format` | `string` | No | `url` (default) or `b64_json` |
| `output_format` | `string` | No | Image encoding such as `png` / `jpeg` / `webp` |
| `watermark` | `boolean` | No | Whether to add a watermark |
| `web_search` | `boolean` | No | Enable web search (default `false`) |
| `ratio` | `string` | No | Aspect ratio, e.g. `16:9`, `3:4` |
| `image` | `string / array` | No | Reference image URL(s) for image-to-image |
| `image_urls` | `array` | No | Reference image URL array |

#### Curl Image Generation Example
```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "dall-e-3",
    "prompt": "一只在太空中漂浮的宇航员猫，写实赛博朋克风格",
    "size": "1024x1024",
    "n": 1
  }'
```

#### Response Example (Returns Image URL)
```json
{
  "created": 1719441600,
  "data": [
    {
      "url": "https://example.com/output/img_abc123.png"
    }
  ]
}
```

### 2. Image Edits
* **Path**: `/v1/images/edits`
* **Method**: `POST`

Supports a base image, optional mask, and prompt for local edits / inpainting.
