#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn f() {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("print in wasm");
}
