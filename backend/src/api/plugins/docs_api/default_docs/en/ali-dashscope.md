# Alibaba Bailian (DashScope) Native API Guide

Alibaba Bailian (DashScope) compatible paths. Use your platform API Key.

### 1. Wanx Video Generation (Submit Video)
* **Path**: `/api/v1/services/aigc/video-generation/video-synthesis`
* **Request Method**: `POST`

#### Request Example
```json
{
  "model": "wanx-v1",
  "input": {
    "prompt": "一只金毛寻回犬在金色的秋天落叶中奔跑"
  },
  "parameters": {
    "resolution": "1280*720",
    "duration": 5
  }
} 
```
Async tasks require header `X-DashScope-Async: enable` (include it in your request if the client does not add it).

### 2. Wanx Image Generation (Submit Image)
* **Path**: `/api/v1/services/aigc/multimodal-generation/generation`
* **Request Method**: `POST`

Similar request shape to video; supports vendor fields such as seed and size.

### 3. Asynchronous Task Status Query
* **Path**: `/api/v1/tasks/{task_id}`
* **Request Method**: `GET`

Both Alibaba Wanx video and image tasks use Bailian's unified asynchronous task ID. You can use the native `task_id` for polling. Usage is settled when the task reaches `succeeded` or `failed`.

### 4. Text Embeddings (Embeddings) and Rerank
* **Embeddings API**: `/compatible-mode/v1/embeddings` (`POST`)
  Supports official Tongyi Qianwen embedding models (e.g., `text-embedding-v4`), billed by the total number of tokens.
* **Document Reranking API (Rerank)**:
  * Compatible path (used for qwen3-rerank, etc.): `/compatible-api/v1/reranks`
  * Native path (used for gte-rerank-v2, etc.): `/api/v1/services/rerank/text-rerank/text-rerank`
