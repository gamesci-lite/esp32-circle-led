fn main() {
    embuild::espidf::sysenv::output();

    // Slint UI 编译: 暂时禁用 — 等屏验证通过后再迭代 Slint 集成.
    // slint_build::compile("src/ui/app.slint").expect("Slint 编译失败 — 检查 src/ui/app.slint");

    // WiFi 凭据: 从 gitignore 的 wifi_secrets.toml 读取, 生成 consts 编入固件。
    // 文件不存在 → 空 SSID → 固件跳过 STA 直接 AP 回退模式。
    let (mut ssid, mut pass, mut user) = (String::new(), String::new(), String::new());
    if let Ok(txt) = std::fs::read_to_string("wifi_secrets.toml") {
        for line in txt.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("SSID") {
                ssid = v.trim_start_matches(['=', ' ']).trim_matches('"').to_string();
            } else if let Some(v) = line.strip_prefix("PASS") {
                pass = v.trim_start_matches(['=', ' ']).trim_matches('"').to_string();
            } else if let Some(v) = line.strip_prefix("USER") {
                user = v.trim_start_matches(['=', ' ']).trim_matches('"').to_string();
            }
        }
    }
    let out = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        std::path::Path::new(&out).join("wifi_secrets.rs"),
        format!(
            "const WIFI_SSID: &str = {:?};\nconst WIFI_PASS: &str = {:?};\nconst WIFI_USER: &str = {:?};\n",
            ssid, pass, user
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=wifi_secrets.toml");
    println!("cargo:rerun-if-changed=src/ui/app.slint");
}
