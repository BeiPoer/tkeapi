# Volcengine Ark Native API Guide

If your client already uses Volcengine Ark request paths, point the Base URL to this platform and use your platform API Key—no need to rewrite payloads to OpenAI format.

### 1. Native Chat & Responses
* **Chat Endpoint**: `/api/v3/chat/completions`
* **Native Responses Endpoint**: `/api/v3/responses`
* **Request Method**: `POST`

Supports Volcengine Ark Request Payload. For parameter specifications, see the [Volcengine Ark Official Documentation](https://www.volcengine.com/docs/82379/1298454).

### 2. Native Image Generation (Image Generations)
* **Endpoint**: `/api/v3/images/generations`
* **Request Method**: `POST`

Perfectly aligned with the Ark Text-to-Image interface, supporting native parameters such as specifying image aspect ratios, intelligent prompt rewriting, and watermarks.

### 3. Native Video Generation Tasks (Video Studio)
* **Submit Task**: `/api/v3/contents/generations/tasks` (`POST`)
* **Query Task Status**: `/api/v3/contents/generations/tasks/{task_id}` (`GET`)
* **Cancel/Delete Task**: `/api/v3/contents/generations/tasks/{task_id}` (`DELETE`)
* **List Task History**: `/api/v3/contents/generations/tasks` (`GET`)

### 4. Text-to-Speech API (TTS)
* **Event Stream Mode (SSE)**: `/api/v3/tts/unidirectional/sse` (`POST`)
* **Non-streaming HTTP Mode**: `/api/v3/tts/unidirectional` (`POST`)

Use header `X-Api-Key: sk-your_token`. Model may be set via `X-Api-Resource-Id` or the `model` field in the body. Response is Volcengine-style JSON (base64 audio frames).
