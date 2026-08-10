# MiniMax Video Generation Example

`MiniMax-H3` supports text-to-video, first/last-frame image-to-video, and multimodal reference-to-video (image / video / audio). Use the OpenAI-compatible async flow: submit then poll. First/last-frame roles must not mix with reference roles.

### 1. Submit Task
* **HTTP Method**: `POST`
* **Request Path**: `https://{{domain}}/v1/video/generations`

#### A. Text-to-Video
```bash
curl -X POST https://{{domain}}/v1/video/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "MiniMax-H3",
    "prompt": "Epic space-opera teaser: a captain stands alone as the fleet jumps away",
    "resolution": "2K",
    "ratio": "16:9",
    "duration": 5
  }'
```

#### B. First / Last Frame Image-to-Video
```bash
curl -X POST https://{{domain}}/v1/video/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "MiniMax-H3",
    "prompt": "Pull focus to the background and add more steam",
    "images": [
      "https://example.com/first_frame.png",
      "https://example.com/last_frame.png"
    ],
    "resolution": "2K",
    "duration": 5
  }'
```

#### C. Reference-to-Video (image + video + audio)
```bash
curl -X POST https://{{domain}}/v1/video/generations \
  -H "Authorization: Bearer sk-your_token_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "MiniMax-H3",
    "prompt": "Character speaks with the reference voice; motion follows the reference video",
    "images": [{ "url": "https://example.com/character.png", "role": "reference_image" }],
    "videos": ["https://example.com/motion_ref.mp4"],
    "audios": ["https://example.com/voice_ref.mp3"],
    "resolution": "2K",
    "duration": 5
  }'
```

### 2. Poll Result
```bash
curl -X GET https://{{domain}}/v1/video/generations/video_task_minimax_001 \
  -H "Authorization: Bearer sk-your_token_here"
```

### 3. Key Parameters
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `model` | `string` | Yes | `MiniMax-H3` |
| `prompt` | `string` | Yes | Text prompt (always required) |
| `images` | `array` | No | 1→first frame, 2→first+last, or explicit `role` |
| `videos` / `audios` | `array` | No | Reference media (reference scenario only) |
| `resolution` | `string` | Yes | `768P` or `2K` |
| `duration` | `integer` | Yes | `4`–`15` seconds |
| `ratio` | `string` | Conditional | Required for T2V (not `adaptive`) |

### 4. Response Example
```json
{
  "id": "video_task_minimax_001",
  "task_id": "video_task_minimax_001",
  "status": "completed",
  "data": [{ "url": "https://example.com/output/minimax_h3.mp4" }]
}
```
