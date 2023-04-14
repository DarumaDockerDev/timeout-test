use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, routing::get,
    BoxError, Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    params, Module, PluginManager, Vm,
};

use std::time::Duration;

async fn wasm_await() -> impl IntoResponse {
    PluginManager::load_from_default_paths();
    let module = Module::from_file(None, "./target/wasm32-wasi/debug/func.wasm").unwrap();

    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()
        .unwrap();
    let mut vm = Vm::new(Some(config))
        .unwrap()
        .register_module(None, module)
        .unwrap();
    let mut wasi_module = vm.wasi_module().unwrap();
    wasi_module.initialize(None, None, None);

    vm.run_func_async(None::<&str>, "f", params!())
        .await
        .unwrap();

    drop(vm);

    ""
}

async fn normal_await() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_secs(2)).await;
    ""
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/normal", get(normal_await))
        .route("/wasm", get(wasm_await))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(1)),
        );

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8094".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        println!("------ [Error handler] timeout ---------");
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        println!("------ [Error handler] other error ---------");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}
