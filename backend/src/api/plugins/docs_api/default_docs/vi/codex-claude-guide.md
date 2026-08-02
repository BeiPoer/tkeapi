# Hướng dẫn tích hợp Codex & Claude Code

Hướng dẫn chi tiết cấu hình giao diện TokensByte gateway, proxy routing và truy vấn số dư tài khoản cho các client AI Agent (như Codex và Claude Code).

---

## 1. Tích hợp Claude Code

### 1. Cấu hình giao diện và mô hình
* **Các mô hình Claude**: Hỗ trợ giao diện chuẩn `/v1/messages`.

  ![Cấu hình Claude Code Messages](/assets/docs/codex-claude/image1.png)

* **Các mô hình cao cấp (GPT-5.6 / Grok-4.5)**: Khuyên dùng giao diện `/v1/responses`.

  ![Cấu hình Claude Code Responses](/assets/docs/codex-claude/image2.png)

  > [!TIP]
  > Nhấp vào "Lấy danh sách mô hình" trong cài đặt client để hiển thị menu thả xuống.

### 2. Cài đặt Proxy và Routing
* Sau khi thêm kênh mô hình, hãy đảm bảo **bật Proxy**:

  ![Bật Proxy](/assets/docs/codex-claude/image3.png)

* Kiểm tra xem **Smart Routing** đã được bật chưa:

  ![Kiểm tra Smart Routing](/assets/docs/codex-claude/image4.png)

### 3. Cấu hình truy vấn số dư
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
        planName: "Số dư tài khoản",
        remaining: response.remain_balance,
        used: response.used_balance,
        total: (response.remain_balance + response.used_balance),
        unit: "CNY"
      };
    }
    return {
      isValid: false,
      invalidMessage: response.message || "Truy vấn thất bại"
    };
  }
})
```

---

## 2. Tích hợp Codex

### 1. Cấu hình giao diện và mô hình
* **Mô hình thế hệ mới (GPT-5.6 / Grok-4.5)**: Khuyên dùng giao diện `/v1/responses`.

  ![Cấu hình Codex Responses](/assets/docs/codex-claude/image5.png)

* **Mô hình chung (DeepSeek, GLM, v.v.)**: Có thể sử dụng giao diện chuẩn `/v1/chat/completions` (Chat).

  ![Cấu hình Codex Chat 1](/assets/docs/codex-claude/image6.png)

  ![Cấu hình Codex Chat 2](/assets/docs/codex-claude/image7.png)

> [!WARNING]
> **Lưu ý về các mô hình Claude**:
> Client Codex truyền tham số không tương thích với Claude, do đó **Codex không thể kết nối với mô hình Claude bằng giao diện Chat**.

### 2. Cấu hình truy vấn số dư
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
        planName: "Số dư tài khoản",
        remaining: response.remain_balance,
        used: response.used_balance,
        total: (response.remain_balance + response.used_balance),
        unit: "USD"
      };
    }
    return {
      isValid: false,
      invalidMessage: response.message || "Truy vấn thất bại"
    };
  }
})
```

---

## 3. Câu hỏi thường gặp (QA)

### 1. Tại sao mức tiêu thụ cache prompt của AI Agent lại cao?
Các AI Agent như Codex và Claude Code gửi kèm System Prompt dài mặc định.

![Prompt và Cache của Agent](/assets/docs/codex-claude/image8.png)

### 2. Điều kiện kích hoạt cache
Context caching được kích hoạt sau **nhiều lượt hội thoại liên tiếp**.
