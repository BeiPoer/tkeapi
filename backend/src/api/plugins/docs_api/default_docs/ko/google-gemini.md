# Google Gemini 네이티브 API 설명

Google SDK 또는 Gemini 형식을 사용하는 경우, 아래 경로와 본 플랫폼 API Key로 호출하세요.

### 1. 텍스트 생성 (Non-stream)
* **경로**: `/v1beta/models/{model}:generateContent`
* **요청 메서드**: `POST`

### 2. 스트리밍 생성 (Streaming)
* **경로**: `/v1beta/models/{model}:streamGenerateContent`
* **요청 메서드**: `POST`

#### 핵심 요청 페이로드 예시
```json
{
  "contents": [
    {
      "role": "user",
      "parts": [
        {
          "text": "请扮演我的私人旅行助手，规划一份 3 天的京都赏樱路线。"
        }
      ]
    }
  ],
  "systemInstruction": {
    "parts": [
      {
        "text": "你是一个专业的旅行规划师，语气亲切幽默。"
      }
    ]
  },
  "generationConfig": {
    "temperature": 0.4,
    "maxOutputTokens": 2000,
    "responseMimeType": "text/plain"
  }
}
```

#### 인증 방식 (택일)
* 표준 헤더: `Authorization: Bearer sk-your_token`
* Google 헤더: `X-Goog-Api-Key: sk-your_token`
* URL 파라미터: `?key=sk-your_token`
