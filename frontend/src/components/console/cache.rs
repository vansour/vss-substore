use dioxus::prelude::*;

use super::{
    actions,
    state::{CacheDisplay, FeedbackSignals, PendingState, RefreshState},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CachePresentation {
    headline: &'static str,
    summary: &'static str,
    route_note: &'static str,
    action_hint: &'static str,
    tone_class: &'static str,
}

#[component]
pub fn CachePanel(
    username: String,
    cache: CacheDisplay,
    cache_error: Option<String>,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let username_for_refresh = username.clone();
    let username_for_clear = username.clone();
    let mut confirm_clear = use_signal(|| false);
    let presentation = cache_presentation(&cache.state);
    let refresh_pending = (pending.refresh_cache)();
    let clear_pending = (pending.clear_cache)();
    let cache_busy = refresh_pending || clear_pending;
    let refresh_label = if refresh_pending {
        "刷新中…"
    } else {
        cache_refresh_label(&cache.state)
    };
    let clear_label = if clear_pending {
        "清空中…"
    } else if confirm_clear() {
        "请在下方确认"
    } else {
        "清空缓存"
    };

    {
        let username = username.clone();
        use_effect(move || {
            let _ = &username;
            confirm_clear.set(false);
        });
    }

    rsx! {
        article { id: "cache-panel", class: "panel cache-panel-card",
            div { class: "section-head",
                div {
                    p { class: "eyebrow", "缓存" }
                    h2 { "聚合快照" }
                }
                div { class: "badge-row",
                    span { class: "tag {cache.state_class()}", "{cache.state_label()}" }
                    span { class: "tag", "{username}" }
                }
            }
            article { class: "cache-verdict {presentation.tone_class}",
                div { class: "cache-verdict__copy",
                    p { class: "eyebrow", "当前判断" }
                    h3 { "{presentation.headline}" }
                    p { class: "muted", "{presentation.summary}" }
                }
                div { class: "badge-row",
                    span { class: "tag {cache.state_class()}", "{cache.state_label()}" }
                    span { class: "tag", "{presentation.action_hint}" }
                }
            }
            div { class: "cache-fact-grid",
                article { class: "cache-fact",
                    span { class: "stat-kicker", "公开订阅" }
                    strong { "{presentation.route_note}" }
                }
                article { class: "cache-fact",
                    span { class: "stat-kicker", "建议动作" }
                    strong { "{presentation.action_hint}" }
                }
            }
            div { class: "metric-grid cache-metric-grid",
                article { class: "metric-card",
                    span { class: "stat-kicker", "行数" }
                    strong { class: "metric-value", "{cache.line_count}" }
                }
                article { class: "metric-card",
                    span { class: "stat-kicker", "字节" }
                    strong { class: "metric-value", "{cache.body_bytes}" }
                }
                article { class: "metric-card",
                    span { class: "stat-kicker", "生成时间" }
                    strong { class: "metric-value metric-value--small", "{cache.generated_at}" }
                }
                article { class: "metric-card",
                    span { class: "stat-kicker", "过期时间" }
                    strong { class: "metric-value metric-value--small", "{cache.expires_at}" }
                }
            }
            div { class: "button-row cache-panel-card__actions",
                button {
                    class: "button button--primary",
                    "data-testid": "cache-refresh-button",
                    disabled: cache_busy,
                    aria_busy: if refresh_pending { "true" } else { "false" },
                    onclick: move |_| {
                        actions::refresh_cache(
                            username_for_refresh.clone(),
                            pending.refresh_cache,
                            feedback,
                            refresh,
                        )
                    },
                    "{refresh_label}"
                }
                button {
                    class: "button button--ghost",
                    disabled: cache_busy,
                    onclick: move |_| refresh.bump_cache(),
                    "重新加载状态"
                }
                button {
                    class: "button button--danger",
                    disabled: cache_busy,
                    aria_busy: if clear_pending { "true" } else { "false" },
                    onclick: move |_| confirm_clear.set(true),
                    "{clear_label}"
                }
            }
            if confirm_clear() {
                article {
                    class: "confirm-strip confirm-strip--danger",
                    role: "group",
                    "aria-label": "清空缓存确认",
                    div { class: "confirm-strip__copy",
                        p { class: "eyebrow", "危险操作" }
                        strong { "确认清空 {username}" }
                        p { class: "muted", "这会立即删除当前订阅组的聚合快照；下次公开访问或手动刷新时才会重新生成。" }
                    }
                    div { class: "button-row confirm-strip__actions",
                        button {
                            class: "button button--ghost",
                            disabled: cache_busy,
                            onclick: move |_| confirm_clear.set(false),
                            "取消"
                        }
                        button {
                            class: "button button--danger",
                            disabled: cache_busy,
                            aria_busy: if clear_pending { "true" } else { "false" },
                            onclick: move |_| {
                                confirm_clear.set(false);
                                actions::clear_cache(
                                    username_for_clear.clone(),
                                    pending.clear_cache,
                                    feedback,
                                    refresh,
                                );
                            },
                            if clear_pending { "清空中…" } else { "确认清空缓存" }
                        }
                    }
                }
            }
            if let Some(message) = cache_error {
                article { class: "notice notice--error diagnostics-notice cache-panel-card__error",
                    div {
                        strong { "缓存错误" }
                        p { "{message}" }
                    }
                }
            }
        }
    }
}

fn cache_presentation(state: &str) -> CachePresentation {
    match state {
        "fresh" => CachePresentation {
            headline: "快照状态健康",
            summary: "当前订阅组已经有可直接提供的聚合快照，除非上游内容发生变化，否则不需要立刻刷新。",
            route_note: "下次访问会优先命中当前快照",
            action_hint: "按需复查即可",
            tone_class: "cache-verdict--success",
        },
        "expired" | "stale" => CachePresentation {
            headline: "快照已需要处理",
            summary: "当前快照已经过期或处于陈旧状态，公开订阅还能返回旧值，但建议尽快刷新，避免继续提供过期内容。",
            route_note: "公开访问可能继续返回旧快照",
            action_hint: "建议立即刷新",
            tone_class: "cache-verdict--danger",
        },
        "empty" => CachePresentation {
            headline: "还没有可用快照",
            summary: "这个订阅组还没有生成过聚合结果。首次访问公开订阅或手动刷新缓存后，状态才会变成可用。",
            route_note: "首次访问将触发生成流程",
            action_hint: "建议立即生成",
            tone_class: "cache-verdict--cool",
        },
        _ => CachePresentation {
            headline: "缓存状态未知",
            summary: "当前状态没有被前端识别，建议重新读取缓存状态，必要时直接刷新快照。",
            route_note: "暂时无法判断公开订阅会返回什么",
            action_hint: "先重新读取状态",
            tone_class: "cache-verdict--cool",
        },
    }
}

fn cache_refresh_label(state: &str) -> &'static str {
    match state {
        "fresh" => "重新构建快照",
        "expired" | "stale" => "立即刷新快照",
        "empty" => "立即生成快照",
        _ => "刷新缓存",
    }
}

