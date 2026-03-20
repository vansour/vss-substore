mod api;
mod app;
mod components;
mod messages;

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!(
        "{} Web 应用仅面向 wasm32 目标。请使用 `dx serve`，或通过 `--target wasm32-unknown-unknown` 进行构建。",
        submora_core::APP_NAME
    );
}
