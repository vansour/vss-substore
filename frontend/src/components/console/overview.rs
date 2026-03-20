use dioxus::prelude::*;

use super::view::UserSummary;

#[component]
pub fn UserOverview(summary: UserSummary) -> Element {
    let attention_count = summary.attention_count();
    let cache_badge_class = summary.cache_badge_class();
    let attention_badge_class = if attention_count > 0 {
        "tag tag--danger"
    } else {
        "tag tag--success"
    };
    let attention_badge_label = if attention_count > 0 {
        format!("{attention_count} 项待处理")
    } else {
        "当前无待处理项".to_string()
    };

    rsx! {
        article { class: "panel workspace-overview",
            div { class: "workspace-overview__masthead",
                div { class: "workspace-overview__headline",
                    p { class: "eyebrow", "订阅组工作台" }
                    h2 { "{summary.selected_username}" }
                    p { class: "muted", "公开订阅地址 {summary.selected_route} · 当前订阅组已进入编辑与诊断视图。" }
                    div { class: "button-row workspace-overview__actions",
                        a {
                            class: "button button--primary",
                            href: "{summary.selected_route}",
                            target: "_blank",
                            rel: "noreferrer",
                            "打开公开订阅"
                        }
                        a {
                            class: "button button--ghost",
                            href: "#editor-panel",
                            "编辑源链接"
                        }
                        a {
                            class: "button button--ghost",
                            href: "#diagnostics-panel",
                            "查看诊断"
                        }
                    }
                }
                div { class: "workspace-overview__spotlight",
                    div { class: "badge-row workspace-overview__badges",
                        span { class: "{cache_badge_class}", "缓存 {summary.cache_display.state_label()}" }
                        span { class: "{attention_badge_class}", "{attention_badge_label}" }
                    }
                    article { class: "workspace-overview__route-card",
                        p { class: "eyebrow", "公开路由" }
                        code { "{summary.selected_route}" }
                        p { class: "muted", "这个地址就是对外可访问的聚合结果入口，缓存和诊断都会围绕它变化。" }
                    }
                }
            }
            div { class: "workspace-overview__metrics",
                article { class: "workspace-stat",
                    span { class: "stat-kicker", "缓存状态" }
                    strong { class: "metric-value metric-value--small", "{summary.cache_display.state_label()}" }
                }
                article { class: "workspace-stat",
                    span { class: "stat-kicker", "源链接" }
                    strong { class: "metric-value", "{summary.selected_link_count}" }
                }
                article { class: "workspace-stat",
                    span { class: "stat-kicker", "正常" }
                    strong { class: "metric-value", "{summary.success_count}" }
                }
                article { class: "workspace-stat",
                    span { class: "stat-kicker", "需关注" }
                    strong { class: "metric-value", "{attention_count}" }
                }
            }
        }
    }
}
