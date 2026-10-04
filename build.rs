fn main() {
    println!("cargo::rerun-if-changed=assets/logo.ico");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(optimized)");
    println!("cargo::rerun-if-env-changed=OPT_LEVEL");

    if std::env::var("OPT_LEVEL").unwrap_or_default() != "0" {
        println!("cargo::rustc-cfg=optimized");
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os != "windows" {
        return;
    }

    let mut resource = winresource::WindowsResource::new();

    resource
        .set_icon_with_id("assets/logo.ico", "1000")
        .set("InternalName", "swarm");

    if let Err(error) = resource.compile() {
        println!("cargo::warning=the Windows resource was not embedded: {error}");
    }
}
