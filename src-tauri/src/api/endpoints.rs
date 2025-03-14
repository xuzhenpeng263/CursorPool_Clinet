use super::client::get_base_url;
use super::types::*;
use tauri::State;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct BugReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub app_version: String,
    pub os_version: String,
    pub device_model: String,
    pub cursor_version: String,
    pub bug_description: String,
    pub occurrence_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_urls: Option<Vec<String>>,
    pub severity: String,
}

#[tauri::command]
pub async fn check_user(
    client: State<'_, super::client::ApiClient>,
    username: String,
) -> Result<ApiResponse<CheckUserResponse>, String> {
    let response = client
        .0
        .post(format!("{}/user/check", get_base_url()))
        .json(&CheckUserRequest { username })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_code(
    client: State<'_, super::client::ApiClient>,
    username: String,
    is_reset_password: Option<bool>,
) -> Result<ApiResponse<SendCodeResponse>, String> {
    let response = client
        .0
        .post(format!("{}/user/send_code", get_base_url()))
        .json(&SendCodeRequest {
            username,
            is_reset_password,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // 先获取响应文本
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    // 打印响应文本用于调试
    println!("Send code response: {}", response_text);
    // 解析JSON响应
    serde_json::from_str(&response_text).map_err(|e| e.to_string())
    // response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn login(
    client: State<'_, super::client::ApiClient>,
    username: String,
    password: String,
    device_id: String,
    sms_code: Option<String>,
) -> Result<LoginResponse, String> {
    let response = client
        .0
        .post(format!("{}/user/login", get_base_url()))
        .json(&LoginRequest {
            username,
            password,
            device_id,
            sms_code,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let api_response: ApiResponse<LoginResponse> = response.json().await
        .map_err(|e| e.to_string())?;
    
    // 如果状态不是成功, 返回错误
    if api_response.status != "success" {
        return Err(api_response.message);
    }
    
    // 从 ApiResponse 中提取 LoginResponse
    api_response.data.ok_or_else(|| "No login data received".to_string())
}

#[tauri::command]
pub async fn get_user_info(
    client: State<'_, super::client::ApiClient>,
    api_key: String,
) -> Result<ApiResponse<UserInfo>, String> {
    let response = client
        .0
        .get(format!("{}/user/info", get_base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activate(
    client: State<'_, super::client::ApiClient>,
    api_key: String,
    code: String,
) -> Result<ApiResponse<ActivateResponse>, String> {
    let response = client
        .0
        .post(format!("{}/user/activate", get_base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ActivateRequest { code })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn change_password(
    client: State<'_, super::client::ApiClient>,
    api_key: String,
    old_password: String,
    new_password: String,
) -> Result<ApiResponse<LoginResponse>, String> {
    let response = client
        .0
        .post(format!("{}/user/change_password", get_base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChangePasswordRequest {
            old_password,
            new_password,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // 先打印原始响应内容
    let raw_response = response.text().await.map_err(|e| e.to_string())?;
    println!("Raw change password response: {}", raw_response);

    // 尝试解析为JSON
    let result: ApiResponse<LoginResponse> = serde_json::from_str(&raw_response)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(result)
}

#[tauri::command]
pub async fn get_account(
    client: State<'_, super::client::ApiClient>,
    api_key: String,
) -> Result<ApiResponse<AccountDetail>, String> {
    let response = client
        .0
        .get(format!("{}/account/get", get_base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let account_response: ApiResponse<AccountInfo> = response
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    // 只返回需要的字段
    Ok(ApiResponse {
        status: account_response.status,
        message: account_response.message,
        data: account_response.data.map(|account_info| {
            let parts: Vec<&str> = account_info.token.split("%3A%3A").collect();
            AccountDetail {
                email: account_info.email,
                user_id: parts[0].to_string(),
                token: parts[1].to_string(),
            }
        }),
    })
}

#[tauri::command]
pub async fn get_usage(
    client: State<'_, super::client::ApiClient>,
    token: String,
) -> Result<ApiResponse<CursorUsageInfo>, String> {
    let user_id = "user_01000000000000000000000000";
    let response = client
        .0
        .get("https://www.cursor.com/api/usage")
        .header("Cookie", format!("WorkosCursorSessionToken={}%3A%3A{}", user_id, token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let usage_info: CursorUsageInfo = response.json().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        status: "success".to_string(),
        message: "获取使用情况成功".to_string(),
        data: Some(usage_info),
    })
}

#[tauri::command]
pub async fn get_user_info_cursor(
    client: State<'_, super::client::ApiClient>,
    user_id: String,
    token: String,
) -> Result<ApiResponse<CursorUserInfo>, String> {
    let response = client
        .0
        .get("https://www.cursor.com/api/auth/me")
        .header("Cookie", format!("WorkosCursorSessionToken={}%3A%3A{}", user_id, token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let user_info: CursorUserInfo = response.json().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        status: "success".to_string(),
        message: "获取用户信息成功".to_string(),
        data: Some(user_info),
    })
}

#[tauri::command]
pub async fn get_version(
    client: State<'_, super::client::ApiClient>,
) -> Result<ApiResponse<VersionInfo>, String> {
    let response = client
        .0
        .get(format!("{}/version", get_base_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::from_str(&response.text().await.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_public_info(
    client: State<'_, super::client::ApiClient>,
) -> Result<ApiResponse<PublicInfo>, String> {
    let response = client
        .0
        .get(format!("{}/public/info", get_base_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_password(
    client: State<'_, super::client::ApiClient>,
    email: String,
    sms_code: String,
    new_password: String,
) -> Result<ApiResponse<()>, String> {
    let response = client
        .0
        .post(format!("{}/user/reset_password", get_base_url()))
        .json(&ResetPasswordRequest {
            email,
            sms_code,
            new_password,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn report_bug(
    client: State<'_, super::client::ApiClient>,
    severity: String,
    bug_description: String,
    api_key: Option<String>,
    screenshot_urls: Option<Vec<String>>,
    cursor_version: Option<String>,
) -> Result<(), String> {
    // 获取应用版本
    let app_version = env!("CARGO_PKG_VERSION").to_string();
    
    // 获取操作系统信息
    let os_info = os_info::get();
    let os_version = format!("{} {}", os_info.os_type(), os_info.version());
    
    // 获取设备型号
    let device_model = sys_info::hostname()
        .unwrap_or_else(|_| "Unknown".to_string());
    
    // 获取当前时间，ISO 8601 格式
    let occurrence_time = Utc::now().to_rfc3339();
    
    // 获取 Cursor 版本，如果未提供则从数据库获取
    let cursor_version = cursor_version.unwrap_or_else(|| {
        crate::utils::CursorVersion::get_version()
            .unwrap_or_else(|_| "Unknown".to_string())
    });
    
    // 创建请求体
    let report = BugReportRequest {
        api_key,
        app_version,
        os_version,
        device_model,
        cursor_version,
        bug_description,
        occurrence_time,
        screenshot_urls,
        severity,
    };

    // 发送请求
    let _response = client
        .0
        .post(format!("{}/report", get_base_url()))
        .json(&report)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_disclaimer(
    client: State<'_, super::client::ApiClient>,
) -> Result<ApiResponse<DisclaimerResponse>, String> {
    let response = client
        .0
        .get(format!("{}/disclaimer", get_base_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json().await.map_err(|e| e.to_string())
}
