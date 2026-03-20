#[cfg(target_arch = "wasm32")]
pub fn translate_backend_message(message: &str) -> String {
    if let Some(detail) = message.strip_prefix("username: ") {
        return format!("用户名：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("password: ") {
        return format!("密码：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("new_username: ") {
        return format!("新用户名：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("new_password: ") {
        return format!("新密码：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("current_password: ") {
        return format!("当前密码：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("links: ") {
        return format!("链接：{}", translate_detail(detail));
    }
    if let Some(detail) = message.strip_prefix("order: ") {
        return format!("排序：{}", translate_detail(detail));
    }

    translate_detail(message)
}

pub fn translate_diagnostic_detail(detail: &str) -> String {
    translate_detail(detail)
}

pub fn extract_field_validation_error(
    message: &str,
    field_name: &str,
    localized_field_name: &str,
) -> Option<String> {
    message
        .strip_prefix(&format!("{field_name}: "))
        .or_else(|| message.strip_prefix(&format!("{localized_field_name}：")))
        .or_else(|| message.strip_prefix(&format!("{localized_field_name}: ")))
        .map(|detail| detail.trim().to_string())
}

fn translate_detail(detail: &str) -> String {
    if detail == "Please login" {
        return "请先登录".to_string();
    }
    if detail == "invalid username" {
        return "用户名不合法".to_string();
    }
    if detail == "password must be 1-128 characters" {
        return "密码长度必须为 1 到 128 个字符".to_string();
    }
    if detail == "password must include letters, numbers, and symbols" {
        return "密码必须同时包含字母、数字和符号".to_string();
    }
    if detail == "current password is required" {
        return "必须填写当前密码".to_string();
    }
    if detail == "current password is incorrect" {
        return "当前密码不正确".to_string();
    }
    if detail == "username already exists" {
        return "用户名已存在".to_string();
    }
    if detail == "must not be empty" {
        return "不能为空".to_string();
    }
    if detail == "user not found" {
        return "订阅组不存在".to_string();
    }
    if detail == "missing csrf token in session" {
        return "会话中缺少 CSRF 令牌".to_string();
    }
    if detail == "missing csrf token header" {
        return "请求缺少 CSRF 令牌".to_string();
    }
    if detail == "invalid csrf token" {
        return "CSRF 令牌无效".to_string();
    }
    if detail == "Fetch completed successfully" {
        return "抓取成功".to_string();
    }
    if detail == "No fetch attempt recorded yet" {
        return "尚未记录抓取尝试".to_string();
    }

    if let Some(seconds) = detail
        .strip_prefix("too many login attempts, retry in ")
        .and_then(|value| value.strip_suffix('s'))
    {
        return format!("登录尝试过多，请在 {seconds} 秒后重试");
    }
    if let Some(seconds) = detail
        .strip_prefix("too many public requests, retry in ")
        .and_then(|value| value.strip_suffix('s'))
    {
        return format!("公共请求过多，请在 {seconds} 秒后重试");
    }
    if let Some(count) = detail
        .strip_prefix("maximum ")
        .and_then(|value| value.strip_suffix(" users allowed"))
    {
        return format!("最多允许 {count} 个订阅组");
    }
    if let Some(count) = detail
        .strip_prefix("maximum ")
        .and_then(|value| value.strip_suffix(" allowed"))
    {
        return format!("最多允许 {count} 项");
    }
    if let Some(value) = detail.strip_prefix("invalid username: ") {
        return format!("用户名不合法：{value}");
    }
    if let Some(value) = detail.strip_prefix("duplicate username: ") {
        return format!("用户名重复：{value}");
    }
    if detail == "order must include every existing user exactly once" {
        return "排序必须且只能包含每个现有用户一次".to_string();
    }
    if let Some(value) = detail.strip_prefix("invalid url: ") {
        return format!("无效的 URL：{value}");
    }
    if let Some(value) = detail.strip_prefix("unsupported scheme: ") {
        return format!("不支持的协议：{value}");
    }
    if let Some(value) = detail.strip_prefix("missing host: ") {
        return format!("缺少主机名：{value}");
    }
    if let Some(value) = detail.strip_prefix("failed to resolve host: ") {
        return format!("解析主机失败：{value}");
    }
    if let Some(value) = detail.strip_prefix("unsafe target: ") {
        return format!("不安全的目标：{value}");
    }
    if let Some(value) = detail.strip_prefix("redirect missing location header: ") {
        return format!("重定向响应缺少 Location 头：{value}");
    }
    if let Some(value) = detail.strip_prefix("redirect location is not valid utf-8: ") {
        return format!("重定向 Location 不是有效的 UTF-8：{value}");
    }
    if let Some(value) = detail.strip_prefix("invalid redirect target from ") {
        return format!("重定向目标无效：{value}");
    }
    if let Some(value) = detail.strip_prefix("content too large: exceeds ") {
        return format!("内容过大：流式读取时超过 {value}");
    }
    if let Some(value) = detail.strip_prefix("failed to fetch ") {
        return format!("抓取失败：{value}");
    }
    if let Some(value) = detail.strip_prefix("too many redirects while fetching ") {
        return format!("抓取时重定向过多：{value}");
    }
    if let Some(value) = detail.strip_prefix("unexpected response status ") {
        return format!("收到异常响应状态：{value}");
    }
    if let Some(value) = detail.strip_prefix("content too large while fetching ") {
        return format!("抓取内容过大：{value}");
    }

    detail.to_string()
}
