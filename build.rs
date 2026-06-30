// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

fn main() {
    println!("cargo::rustc-check-cfg=cfg(flatpak_build)");

    let config = slint_build::CompilerConfiguration::new().with_bundled_translations("ui/locale");
    slint_build::compile_with_config("ui/app.slint", config).expect("Slint build failed.");

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if std::env::var("FLATPAK_ID").is_ok() {
            println!("cargo:rustc-cfg=flatpak_build");
        }
    }

    #[cfg(all(windows, not(debug_assertions)))]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("resources/logo/windows.ico");
        res.compile().unwrap();
    }
}
