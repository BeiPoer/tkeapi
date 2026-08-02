# Codex & Claude Code 연동 가이드

AI Agent 클라이언트(Codex, Claude Code 등)를 위해 TokensByte 게이트웨이 인터페이스 설정, 프록시 라우팅 및 계정 잔액 조회 설정 방법을 설명합니다.

---

## 1. Claude Code 연동

### 1. 인터페이스 및 모델 설정
* **Claude 시리즈 모델**: 표준 `/v1/messages` 인터페이스를 지원합니다.

  ![Claude Code Messages 설정](/assets/docs/codex-claude/image1.png)

* **고급 모델 (GPT-5.6 / Grok-4.5 등)**: `/v1/responses` 인터페이스 사용을 권장합니다.

  ![Claude Code Responses 설정](/assets/docs/codex-claude/image2.png)

  > [!TIP]
  > 설정에서 "모델 목록 가져오기"를 클릭해야 드롭다운 메뉴가 표시됩니다.

### 2. 프록시 및 라우팅 설정
* 모델 채널 추가 후 **프록시 활성화**를 확인하세요.

  ![프록시 활성화](/assets/docs/codex-claude/image3.png)

* **스마트 라우팅** 활성화 여부를 확인하세요.

  ![스마트 라우팅 확인](/assets/docs/codex-claude/image4.png)

### 3. 잔액 조회 설정
```javascript
({
  request: {
    url: "{{baseUrl}}/v1/user/balance",
    method: "GET",
    headers: {
      "Authorization": "Bearer {{apiKey}}"
    }
  },
  extractor: function (response) {
    if (response.success) {
      return {
        planName: "사용자 계정 잔액",
        remaining: response.remain_balance,
        used: response.used_balance,
        total: (response.remain_balance + response.used_balance),
        unit: "CNY"
      };
    }
    return {
      isValid: false,
      invalidMessage: response.message || "조회 실패"
    };
  }
})
```

---

## 2. Codex 연동

### 1. 인터페이스 및 모델 설정
* **차세대 모델 (GPT-5.6 / Grok-4.5)**: `/v1/responses` 인터페이스를 권장합니다.

  ![Codex Responses 설정](/assets/docs/codex-claude/image5.png)

* **일반 모델 (DeepSeek, GLM 등)**: 표준 `/v1/chat/completions` (Chat) 인터페이스를 사용할 수 있습니다.

  ![Codex Chat 설정 1](/assets/docs/codex-claude/image6.png)

  ![Codex Chat 설정 2](/assets/docs/codex-claude/image7.png)

> [!WARNING]
> **Claude 모델 연동 주의사항**:
> Codex 클라이언트는 Claude와 호환되지 않는 파라미터를 전달하므로 **Codex에서 Chat 인터페이스로 Claude 모델에 접속할 수 없습니다**.

### 2. 잔액 조회 설정
```javascript
({
  request: {
    url: "{{baseUrl}}/user/balance",
    method: "GET",
    headers: {
      "Authorization": "Bearer {{apiKey}}"
    }
  },
  extractor: function (response) {
    if (response.success) {
      return {
        planName: "사용자 계정 잔액",
        remaining: response.remain_balance,
        used: response.used_balance,
        total: (response.remain_balance + response.used_balance),
        unit: "USD"
      };
    }
    return {
      isValid: false,
      invalidMessage: response.message || "조회 실패"
    };
  }
})
```

---

## 3. 자주 묻는 질문 (QA)

### 1. Agent 클라이언트의 프롬프트 캐시 사용량이 높은 이유
Codex 및 Claude Code와 같은 Agent는 긴 시스템 프롬프트와 도구 정의를 기본적으로 전송합니다.

![Agent 시스템 프롬프트 및 캐시](/assets/docs/codex-claude/image8.png)

### 2. 캐시 적중 조건
컨텍스트 캐싱은 **다중 대화** 후에 적용됩니다.
