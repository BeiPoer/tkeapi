# MiniMax Image Generation Example

MiniMax `image-01` / `image-01-live` are available via the OpenAI-compatible `POST /v1/images/generations` endpoint. Follow the examples below to integrate.

### 1. Base URL & Endpoint
* **HTTP Method**: `POST`
* **Request Path**: `https://{{domain}}/v1/images/generations`

### 2. Text-to-Image
```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "image-01",
    "prompt": "A man in a white t-shirt, full-body, Venice Beach sign background, 90s documentary fashion, film grain",
    "ratio": "16:9",
    "n": 2,
    "prompt_optimizer": true,
    "response_format": "url"
  }'
```

### 3. Image-to-Image / Subject Reference
```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "image-01",
    "prompt": "Keep the character identity, looking out from a library window, cinematic light",
    "image_urls": ["https://example.com/assets/character_face.jpg"],
    "ratio": "16:9",
    "n": 1
  }'
```

### 4. Request Parameters
| Parameter | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `model` | `string` | Yes | - | `image-01` or `image-01-live`. |
| `prompt` | `string` | Yes | - | Image description (up to ~1500 chars). |
| `ratio` / `aspect_ratio` | `string` | No | `1:1` | Aspect ratio; `21:9` only for `image-01`. |
| `image` / `image_urls` | `string / array` | No | - | Reference image(s) for subject. |
| `subject_reference` | `array` | No | - | Subject refs: `{type:"character", image_file}`. |
| `style` | `object` | No | - | `image-01-live` only (`style_type` + `style_weight`). |
| `n` | `integer` | No | `1` | Count `[1, 9]`. |
| `prompt_optimizer` | `boolean` | No | `false` | Auto-optimize prompt. |
| `response_format` | `string` | No | `url` | `url` or `b64_json`. |

### 5. Response Example (200 OK)
```json
{
  "created": 1719441600,
  "data": [{ "url": "https://example.com/output/img_minimax_1.png" }]
}
```
