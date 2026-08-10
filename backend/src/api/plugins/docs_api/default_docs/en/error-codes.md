# Common Error Codes and Troubleshooting

When using the API gateway to access large model services, if a request exception occurs, the gateway will return the error to the client via the corresponding HTTP status code along with a JSON error response body that complies with OpenAI standards.

### 1. Unified Error Response Format
When a request fails, the gateway always returns a standard JSON error body:
```json
{
  "error": {
    "message": "错误原因详细描述...",
    "type": "invalid_request_error",
    "code": "context_length_exceeded",
    "param": null
  }
}
```

### 2. Status Codes and Troubleshooting Guide

* **400 Bad Request (Invalid Request Format)**
  * **Possible Causes**: The request payload is not valid JSON; required parameters (such as `model` or `messages`) are missing; parameter types are incorrect.
  * **Troubleshooting**: Check the HTTP request Body sent, align with the vendor's standard parameters (such as `max_tokens`, etc.), and check field spelling.

* **401 Unauthorized (Unauthorized/Authentication Failed)**
  * **Possible Causes**: The request header does not contain `Authorization` or the Bearer Token is missing; the API key (Token) is invalid or has been deleted by the system; the Token string contains spaces, line breaks, or extra suffixes.
  * **Troubleshooting**: Confirm the request header format is `Authorization: Bearer sk-xxxxx` and check if the token is activated in the admin backend.

* **403 Forbidden (No Permission/Quota Exhausted)**
  * **Possible Causes**: The token's available quota or user's available balance is exhausted; the current token does not have permission to invoke the requested model; the token has been disabled by the administrator or system.
  * **Troubleshooting**: Log into the system frontend to view the token balance; check the "Available Model List" in the token list to see if it includes the currently requested model.

* **404 Not Found (Endpoint or Route Does Not Exist)**
  * **Possible Causes**: The requested URL path has a spelling error; the model has no active channel, or all channels are disabled.
  * **Troubleshooting**: Check the API path (such as `/v1/chat/completions`); confirm the model is associated with active channels in the backend.

* **429 Too Many Requests (Rate Limit Triggered)**
  * **Possible Causes**: Token rate limits (RPM / TPM) or server-side rate limiting was triggered.
  * **Troubleshooting**: Implement exponential backoff retry logic in your code; confirm the token's rate limit settings or contact the administrator to increase the rate limit.

* **500 Internal Error (Gateway Internal Exception)**
  * **Possible Causes**: The gateway database connection is disconnected or timed out; an unhandled code panic occurred within the platform.
  * **Troubleshooting**: Contact the system administrator and check the backend service container logs to locate the cause of the exception.

* **502 Bad Gateway (Service Temporarily Unavailable)**
  * **Possible Causes**: Downstream connection timeout, network interruption, or temporary unavailability.
  * **Troubleshooting**: Check the `message` in the JSON response; retry later or contact the administrator.

* **504 Gateway Timeout (Gateway Response Timeout)**
  * **Possible Causes**: The requested model generation takes a very long time, resulting in an HTTP connection timeout.
  * **Troubleshooting**: For highly time-consuming tasks such as video generation or super-resolution enhancement, submit them using asynchronous interfaces (such as `/v1/video/generations`) and then poll the task status interface to retrieve results.

### 3. Notes
1. **Automatic retry**: With multiple channels configured, brief failures may be retried on another route transparently.
2. **Logs**: Duration, billing, and related details are available in your usage logs.
