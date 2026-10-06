fn main() {
    println!("cargo:rerun-if-env-changed=VULKAN_SDK");
    let sdk = std::env::var("VULKAN_SDK").expect("VULKAN_SDK non impostata: vedi AGENTS.md");
    println!("cargo:rustc-link-search=native={sdk}/Lib");
}
