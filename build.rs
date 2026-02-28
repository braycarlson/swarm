#[cfg(windows)]
fn main() {
    let mut resource = winres::WindowsResource::new();

    resource
        .set_icon_with_id("assets/logo.ico", "1000")
        .set("InternalName", "swarm");

    resource.compile().expect("Resource could not be compiled.");
}

#[cfg(not(windows))]
fn main() {}
