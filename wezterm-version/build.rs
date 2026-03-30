fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // If a file named `.tag` is present, we'll take its contents for the
    // version number that we report in wezterm -h.
    let mut ci_tag = String::new();
    if let Ok(tag) = std::fs::read("../.tag") {
        if let Ok(s) = String::from_utf8(tag) {
            ci_tag = s.trim().to_string();
            println!("cargo:rerun-if-changed=../.tag");
        }
    } else {
        // Otherwise we'll derive it from the git information

        if let Ok(repo) = git2::Repository::discover(".") {
            let git_dir = repo.path().to_path_buf();
            let common_dir = repo.commondir().to_path_buf();
            let work_dir = repo
                .workdir()
                .map(|path| path.to_path_buf())
                .unwrap_or_else(|| {
                    git_dir
                        .parent()
                        .map(|path| path.to_path_buf())
                        .unwrap_or_else(|| git_dir.clone())
                });

            for path in [
                git_dir.join("HEAD"),
                common_dir.join("HEAD"),
                common_dir.join("packed-refs"),
            ] {
                if path.exists() {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }

            if let Ok(ref_head) = repo.find_reference("HEAD") {
                if let Ok(resolved) = ref_head.resolve() {
                    if let Some(name) = resolved.name() {
                        let path = common_dir.join(name);
                        if path.exists() {
                            println!("cargo:rerun-if-changed={}", path.display());
                        }
                    }
                }
            }

            if let Ok(output) = std::process::Command::new("git")
                .current_dir(&work_dir)
                .args(&[
                    "-c",
                    "core.abbrev=8",
                    "show",
                    "-s",
                    "--format=%cd-%h",
                    "--date=format:%Y%m%d-%H%M%S",
                ])
                .output()
            {
                let info = String::from_utf8_lossy(&output.stdout);
                ci_tag = info.trim().to_string();
            }
        }
    }

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=WEZTERM_TARGET_TRIPLE={}", target);
    println!("cargo:rustc-env=WEZTERM_CI_TAG={}", ci_tag);
}
