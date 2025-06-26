use embed_resource::CompilationResult;


fn main() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let rc_path = current_dir.join("ui/asserts/resources.rc");
    slint_build::compile("ui/main.slint").unwrap();
    if cfg!(target_os = "windows") {
        match embed_resource::compile(rc_path.as_os_str(), embed_resource::NONE){
            CompilationResult::Ok => {println!("Compilation resource successful")},
            _ => println!("Compilation resource failed"),
        }
    }
}
