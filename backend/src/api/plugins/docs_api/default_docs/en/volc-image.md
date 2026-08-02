# Volcengine doubao-seedream Image Generation Example

doubao-seedream-5-0-260128 is the next-generation flagship image generation model launched by ByteDance's Jimeng AI. It offers state-of-the-art Chinese prompt comprehension and exquisite image detail.

### 1. Base URL & Endpoint
* **HTTP Method**: `POST`
* **Request Path**: `https://{{domain}}/v1/images/generations`

### 2. Code Example
```bash
curl -X POST https://{{domain}}/v1/images/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "doubao-seedream-5-0-260128",
    "prompt": "A little Shiba Inu wearing a red Tang suit, sitting under red lanterns at the gate, national tide illustration style, festive and warm",
    "size": "1024x1024",
    "n": 1,
    "response_format": "url"
  }'
```

### 3. Request Parameters
| Parameter | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `model` | `string` | Yes | - | Name of the model. Use `doubao-seedream-5-0-260128`. |
| `prompt` | `string` | Yes | - | Prompt describing the image (superb Chinese support). |
| `size` | `string` | No | `1024x1024` | Image dimensions: `1024x1024`, `720x1280`, `1280x720`, etc. |
| `response_format` | `string` | No | `url` | Response format: `url` or `b64_json`. |
| `web_search` | `boolean` | No | `false` | Enable web search. Use this OpenAI-style boolean; — the gateway converts it for Volcengine. |

### 4. Response Example (200 OK)
```json
{
  "created": 1719441600,
  "data": [
    {
      "url": "https://example.com/output/img_seedream_dog.png"
    }
  ]
}
```
