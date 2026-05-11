fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=rabbit-app.rc");
        println!("cargo:rerun-if-changed=../../resources/icon.ico");

        embed_resource::compile("rabbit-app.rc", embed_resource::NONE).manifest_optional().ok();
    }
}
