use std::process::Command;

fn main() {
    // In CI, GITHUB_SHA is injected by the workflow so we don't need to shell out.
    let hash = std::env::var("GITHUB_SHA")
        .map(|sha| sha[..sha.len().min(7)].to_string())
        .unwrap_or_else(|_| {
            Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .and_then(|o| if o.status.success() { Some(o.stdout) } else { None })
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    println!("cargo:rustc-env=GIT_HASH={hash}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
