fn main() {
    // Embed Windows manifest for administrator privileges
    #[cfg(windows)]
    {
        embed_resource::compile("windows/app.rc", embed_resource::NONE);
    }
    
    tauri_build::build()
}
