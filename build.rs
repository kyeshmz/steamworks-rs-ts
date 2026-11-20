extern crate napi_build;

fn main() {
    napi_build::setup();
    cc::Build::new()
        .cpp(true)
        .std("c++14")
        .file("native/server_browser.cpp")
        .include("native/valve")
        .compile("server_browser");
    println!("cargo:rerun-if-changed=native");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
