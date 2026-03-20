use dioxus::prelude::*;
use submora_shared::users::{LinkDiagnostic, UserDiagnosticsResponse};

use crate::messages::translate_diagnostic_detail;

use super::state::format_timestamp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticFilter {
    All,
    Attention,
    Error,
    Blocked,
    Pending,
    Success,
}

#[component]
pub fn DiagnosticsPanel(
    diagnostics: Option<UserDiagnosticsResponse>,
    diagnostics_error: Option<String>,
    success_count: usize,
    error_count: usize,
    blocked_count: usize,
    pending_count: usize,
) -> Element {
    let mut selected_filter = use_signal(|| DiagnosticFilter::Attention);
    let mut show_success_archive = use_signal(|| false);
    let diagnostics_list = diagnostics
        .clone()
        .map(|payload| sorted_diagnostics(payload.diagnostics))
        .unwrap_or_default();
    let attention_diagnostics = diagnostics_list
        .iter()
        .filter(|diagnostic| is_attention_status(&diagnostic.status))
        .cloned()
        .collect::<Vec<_>>();
    let success_diagnostics = diagnostics_list
        .iter()
        .filter(|diagnostic| diagnostic.status == "success")
        .cloned()
        .collect::<Vec<_>>();
    let attention_count = error_count + blocked_count + pending_count;
    let total_count = diagnostics_list.len();
    let summary = diagnostics_summary(
        attention_count,
        success_count,
        total_count,
        selected_filter(),
    );
    let visible_diagnostics = diagnostics_list
        .iter()
        .filter(|diagnostic| matches_filter(selected_filter(), diagnostic))
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        article { id: "diagnostics-panel", class: "panel diagnostics-panel-card",
            div { class: "section-head",
                div {
                    p { class: "eyebrow", "诊断" }
                    h2 { "源抓取" }
                }
                div { class: "badge-row",
                    span { class: "tag tag--success", "{success_count} 成功" }
                    span { class: "tag tag--danger", "{error_count} 错误" }
                    span { class: "tag tag--danger", "{blocked_count} 已拦截" }
                    span { class: "tag tag--cool", "{pending_count} 待处理" }
                }
            }
            if diagnostics.is_some() && !diagnostics_list.is_empty() {
                article { class: "diagnostics-summary {summary.tone_class}",
                    div { class: "diagnostics-summary__copy",
                        p { class: "eyebrow", "优先处理" }
                        h3 { "{summary.headline}" }
                        p { class: "muted", "{summary.note}" }
                    }
                    span { class: "tag", "当前筛选：{selected_filter().label()}" }
                }
                    div { class: "diagnostic-filter-row",
                        for filter in DiagnosticFilter::all() {
                            button {
                                "data-testid": "{filter.test_id()}",
                                class: diagnostic_filter_class(selected_filter() == filter),
                                r#type: "button",
                                aria_pressed: if selected_filter() == filter { "true" } else { "false" },
                                onclick: move |_| selected_filter.set(filter),
                                "{filter.label()} ({filter_count(&diagnostics_list, filter)})"
                            }
                    }
                }
            }
            if let Some(message) = diagnostics_error {
                article { class: "notice notice--error diagnostics-notice",
                    div {
                        strong { "诊断错误" }
                        p { "{message}" }
                    }
                }
            } else if diagnostics.is_some() {
                if diagnostics_list.is_empty() {
                    div { class: "empty-state empty-state--compact",
                        strong { "暂无诊断数据" }
                        p { "保存源链接后，访问公共路由或手动刷新缓存即可生成诊断。" }
                    }
                } else if selected_filter() == DiagnosticFilter::Attention && attention_diagnostics.is_empty() {
                    article { class: "diagnostics-summary diagnostics-summary--success",
                        div { class: "diagnostics-summary__copy",
                            p { class: "eyebrow", "当前状态" }
                            h3 { "当前没有需要处理的抓取问题" }
                            p { class: "muted", "所有已完成抓取的源目前都处于成功状态。你仍然可以在下方展开成功记录做复查。" }
                        }
                        span { class: "tag tag--success", "{success_diagnostics.len()} 条成功" }
                    }
                } else if visible_diagnostics.is_empty() {
                    div { class: "empty-state empty-state--compact diagnostics-empty-filtered",
                        strong { "当前筛选下没有记录" }
                        p { "切换筛选条件，或先处理错误、已拦截和待处理项。" }
                    }
                } else {
                    div { class: "diagnostics-list",
                        for diagnostic in visible_diagnostics {
                            DiagnosticCard { diagnostic }
                        }
                    }
                }
                if selected_filter() == DiagnosticFilter::Attention && !success_diagnostics.is_empty() {
                    article { class: "diagnostics-archive",
                        div { class: "diagnostics-archive__head",
                            div {
                                p { class: "eyebrow", "成功记录" }
                                h3 { "健康项归档" }
                                p { class: "muted", "默认不展开成功项，避免它们干扰当前需要处理的错误、拦截和待处理记录。" }
                            }
                            div { class: "button-row diagnostics-archive__actions",
                                span { class: "tag tag--success", "{success_diagnostics.len()} 条成功" }
                                button {
                                    class: "button button--ghost diagnostic-filter",
                                    r#type: "button",
                                    aria_pressed: if show_success_archive() { "true" } else { "false" },
                                    onclick: move |_| show_success_archive.set(!show_success_archive()),
                                    if show_success_archive() { "收起成功记录" } else { "展开成功记录" }
                                }
                            }
                        }
                        if show_success_archive() {
                            div { class: "diagnostics-archive__list",
                                for diagnostic in success_diagnostics.clone() {
                                    SuccessArchiveRow { diagnostic }
                                }
                            }
                        }
                    }
                }
            } else {
                p { class: "muted", "正在加载诊断..." }
            }
        }
    }
}

impl DiagnosticFilter {
    const fn all() -> [Self; 6] {
        [
            Self::All,
            Self::Attention,
            Self::Error,
            Self::Blocked,
            Self::Pending,
            Self::Success,
        ]
    }

    const fn label(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Attention => "需关注",
            Self::Error => "错误",
            Self::Blocked => "已拦截",
            Self::Pending => "待处理",
            Self::Success => "成功",
        }
    }

    const fn test_id(self) -> &'static str {
        match self {
            Self::All => "diagnostic-filter-all",
            Self::Attention => "diagnostic-filter-attention",
            Self::Error => "diagnostic-filter-error",
            Self::Blocked => "diagnostic-filter-blocked",
            Self::Pending => "diagnostic-filter-pending",
            Self::Success => "diagnostic-filter-success",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiagnosticsSummary {
    headline: String,
    note: String,
    tone_class: &'static str,
}

#[component]
fn DiagnosticCard(diagnostic: LinkDiagnostic) -> Element {
    let status_class = diagnostic_status_class(&diagnostic.status);
    let card_class = format!("diagnostic-card diagnostic-card--{}", diagnostic.status);
    let detail = diagnostic
        .detail
        .as_deref()
        .map(translate_diagnostic_detail)
        .unwrap_or_else(|| "未记录诊断详情".to_string());
    let http_status = diagnostic
        .http_status
        .map(|value| format!("HTTP {value}"))
        .unwrap_or_else(|| "无状态".to_string());
    let content_type = diagnostic
        .content_type
        .clone()
        .unwrap_or_else(|| "未知内容类型".to_string());
    let body_bytes = diagnostic
        .body_bytes
        .map(|value| format!("{value} 字节"))
        .unwrap_or_else(|| "大小未知".to_string());
    let fetched_at = format_timestamp(diagnostic.fetched_at, "尚未抓取");
    let redirect_label = if diagnostic.redirect_count == 1 {
        "1 次重定向".to_string()
    } else {
        format!("{} 次重定向", diagnostic.redirect_count)
    };
    let body_kind = if diagnostic.is_html {
        "HTML 已归一化"
    } else {
        "纯文本"
    };
    let status_label = diagnostic_status_label(&diagnostic.status);
    let attention_hint = attention_count_hint(&diagnostic.status);

    rsx! {
        article { class: "{card_class}",
            div { class: "diagnostic-card__head",
                div { class: "diagnostic-card__identity",
                    code { class: "diagnostic-url", "{diagnostic.url}" }
                    p { class: "muted diagnostic-card__timestamp", "更新于 {fetched_at}" }
                }
                span { class: "diagnostic-status {status_class}", "{status_label}" }
            }
            div { class: "diagnostic-detail",
                p { class: "muted", "{detail}" }
            }
            div { class: "diagnostic-chip-row",
                span { class: "diagnostic-chip", "{http_status}" }
                span { class: "diagnostic-chip", "{content_type}" }
                span { class: "diagnostic-chip", "{body_bytes}" }
                span { class: "diagnostic-chip", "{redirect_label}" }
                span { class: "diagnostic-chip", "{body_kind}" }
            }
            if let Some(attention_hint) = attention_hint {
                p { class: "muted", "{attention_hint}" }
            }
        }
    }
}

#[component]
fn SuccessArchiveRow(diagnostic: LinkDiagnostic) -> Element {
    let fetched_at = format_timestamp(diagnostic.fetched_at, "尚未抓取");
    let content_type = diagnostic
        .content_type
        .clone()
        .unwrap_or_else(|| "未知内容类型".to_string());
    let body_bytes = diagnostic
        .body_bytes
        .map(|value| format!("{value} 字节"))
        .unwrap_or_else(|| "大小未知".to_string());

    rsx! {
        article { class: "diagnostics-archive-item",
            div { class: "diagnostics-archive-item__main",
                code { class: "diagnostic-url", "{diagnostic.url}" }
                p { class: "muted", "最近成功抓取于 {fetched_at}" }
            }
            div { class: "badge-row diagnostics-archive-item__meta",
                span { class: "tag tag--success", "成功" }
                span { class: "tag", "{content_type}" }
                span { class: "tag", "{body_bytes}" }
            }
        }
    }
}

fn sorted_diagnostics(mut diagnostics: Vec<LinkDiagnostic>) -> Vec<LinkDiagnostic> {
    diagnostics.sort_by(|left, right| {
        diagnostic_priority(&left.status)
            .cmp(&diagnostic_priority(&right.status))
            .then_with(|| right.fetched_at.cmp(&left.fetched_at))
            .then_with(|| left.url.cmp(&right.url))
    });
    diagnostics
}

fn diagnostic_priority(status: &str) -> u8 {
    match status {
        "error" => 0,
        "blocked" => 1,
        "pending" => 2,
        "success" => 3,
        _ => 4,
    }
}

fn is_attention_status(status: &str) -> bool {
    matches!(status, "error" | "blocked" | "pending")
}

fn matches_filter(filter: DiagnosticFilter, diagnostic: &LinkDiagnostic) -> bool {
    match filter {
        DiagnosticFilter::All => true,
        DiagnosticFilter::Attention => is_attention_status(&diagnostic.status),
        DiagnosticFilter::Error => diagnostic.status == "error",
        DiagnosticFilter::Blocked => diagnostic.status == "blocked",
        DiagnosticFilter::Pending => diagnostic.status == "pending",
        DiagnosticFilter::Success => diagnostic.status == "success",
    }
}

fn filter_count(diagnostics: &[LinkDiagnostic], filter: DiagnosticFilter) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| matches_filter(filter, diagnostic))
        .count()
}

fn diagnostics_summary(
    attention_count: usize,
    success_count: usize,
    total_count: usize,
    selected_filter: DiagnosticFilter,
) -> DiagnosticsSummary {
    if total_count == 0 {
        return DiagnosticsSummary {
            headline: "等待首次抓取".to_string(),
            note: "保存源链接后，访问公开订阅或手动刷新缓存即可生成第一批诊断。".to_string(),
            tone_class: "diagnostics-summary--cool",
        };
    }

    if attention_count > 0 {
        return DiagnosticsSummary {
            headline: format!("{attention_count} 条记录需要优先处理"),
            note: format!(
                "错误、已拦截和待处理项已经自动排到前面；当前筛选为“{}”。",
                selected_filter.label()
            ),
            tone_class: "diagnostics-summary--danger",
        };
    }

    DiagnosticsSummary {
        headline: format!("{success_count} 条记录已全部成功"),
        note: "当前所有源都已成功抓取，下面的详情仅用于复查响应类型和更新时间。".to_string(),
        tone_class: "diagnostics-summary--success",
    }
}

fn diagnostic_filter_class(active: bool) -> &'static str {
    if active {
        "button button--ghost diagnostic-filter diagnostic-filter--active"
    } else {
        "button button--ghost diagnostic-filter"
    }
}

fn attention_count_hint(status: &str) -> Option<&'static str> {
    match status {
        "blocked" => Some("目标被安全策略拦截，没有进入公共输出。"),
        "pending" => Some("还没有完成抓取，等待首次访问或手动刷新缓存。"),
        "error" => Some("抓取失败，公共输出会跳过这条内容。"),
        _ => None,
    }
}

fn diagnostic_status_label(status: &str) -> &'static str {
    match status {
        "success" => "成功",
        "blocked" => "已拦截",
        "pending" => "待处理",
        _ => "错误",
    }
}

fn diagnostic_status_class(status: &str) -> &'static str {
    match status {
        "success" => "diagnostic-status--success",
        "blocked" => "diagnostic-status--blocked",
        "pending" => "diagnostic-status--pending",
        _ => "diagnostic-status--error",
    }
}

