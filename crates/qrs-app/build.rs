//! The Windows exe's resources: the app icon (Explorer, the taskbar, shortcuts) and its version information.

use std::{path::Path, process::Command};

#[cfg_attr(not(windows), allow(clippy::needless_return))] // the early return is only needless off Windows
fn main() {
    println!("cargo:rerun-if-changed=../../assets/icons/rsplay.ico");

    let root = Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .current_dir(&root)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|hash| hash.trim().to_owned())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());
    println!(
        "cargo:rustc-env=QUICKRS_BUILD_DATE={}",
        chrono::Local::now().format("%Y-%m-%d %H:%M %Z")
    );
    println!("cargo:rustc-env=QUICKRS_GIT_HASH={git_hash}");

    if let Ok(head) = Command::new("git")
        .args(["rev-parse", "--git-path", "HEAD"])
        .current_dir(&root)
        .output()
        && head.status.success()
        && let Ok(head) = String::from_utf8(head.stdout)
    {
        println!("cargo:rerun-if-changed={}", root.join(head.trim()).display());
        if let Ok(reference) = Command::new("git")
            .args(["symbolic-ref", "--quiet", "HEAD"])
            .current_dir(&root)
            .output()
            && reference.status.success()
            && let Ok(reference) = String::from_utf8(reference.stdout)
        {
            let reference = reference.trim();
            if let Ok(path) = Command::new("git")
                .args(["rev-parse", "--git-path", reference])
                .current_dir(&root)
                .output()
                && path.status.success()
                && let Ok(path) = String::from_utf8(path.stdout)
            {
                println!("cargo:rerun-if-changed={}", root.join(path.trim()).display());
            }
        }
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icons/rsplay.ico")
            .set("ProductName", "quickRS Bristol 2026")
            .set(
                "FileDescription",
                "quickRS Bristol 2026: RS song library and player",
            )
            .set("OriginalFilename", "quickrs.exe")
            .set("InternalName", "quickrs");
        if let Err(e) = res.compile() {
            // no Windows SDK resource compiler: the exe works, just without its icon in Explorer
            println!("cargo:warning=the exe has no icon resource: {e}");
        }
    }
}
