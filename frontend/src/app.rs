use dioxus::prelude::*;

use crate::components::{console::AdminConsole, shell::AppShell};

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/login")]
    Login {},
    #[route("/users/:username")]
    UserDetail { username: String },
    #[route("/account")]
    Account {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
pub fn App() -> Element {
    let _app_stylesheet = asset!(
        "/assets/app.css",
        AssetOptions::css().with_static_head(true)
    );
    let _font_assets = asset!("/assets/fonts", AssetOptions::folder());
    let _style_assets = asset!("/assets/styles", AssetOptions::folder());

    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! { AdminConsole { mode: "dashboard", route_user: None } }
}

#[component]
fn Login() -> Element {
    rsx! { AdminConsole { mode: "login", route_user: None } }
}

#[component]
fn UserDetail(username: String) -> Element {
    rsx! { AdminConsole { mode: "user", route_user: Some(username) } }
}

#[component]
fn Account() -> Element {
    rsx! { AdminConsole { mode: "account", route_user: None } }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let route = format!("/{}", segments.join("/"));

    rsx! {
        AppShell {
            title: "页面未找到".to_string(),
            summary: format!("没有为 {route} 注册 Dioxus 路由。"),
            compact: false,
            active_mode: None,
            selected_user: None,
            article { class: "panel",
                div { class: "section-head",
                    div {
                        h2 { "该路由由其他层处理" }
                        p { class: "muted", "管理控制台只接管显式声明的 Dioxus 页面。" }
                    }
                    span { class: "tag", "{route}" }
                }
                p { class: "panel-copy",
                    "公共订阅路由 "
                    code { {"/{username}".to_string()} }
                    " 仍由 Axum 处理，因此聚合后的文本响应不会经过 Dioxus 路由。"
                }
            }
        }
    }
}
