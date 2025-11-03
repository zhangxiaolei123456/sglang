use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only regenerate if the proto file changes
    println!("cargo:rerun-if-changed=src/proto/sglang_scheduler.proto");
    println!("cargo:rerun-if-changed=pyproject.toml");

    // Configure tonic-prost-build for gRPC code generation
    tonic_prost_build::configure()
        // Generate both client and server code
        .build_server(true)
        .build_client(true)
        // Allow proto3 optional fields
        .protoc_arg("--experimental_allow_proto3_optional")
        // Compile the proto file
        .compile_protos(&["src/proto/sglang_scheduler.proto"], &["src/proto"])?;

    println!("cargo:warning=Protobuf compilation completed successfully");

    // Read version and project name from pyproject.toml
    let version = read_version_from_pyproject("version")?;
    let project_name = read_version_from_pyproject("name")?;
    println!("cargo:rustc-env=SG_ROUTER_VERSION={}", version);
    println!("cargo:rustc-env=SG_ROUTER_PROJECT_NAME={}", project_name);

    // Generate build time (UTC)
    let build_time = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=SG_ROUTER_BUILD_TIME={}", build_time);

    // Try to get Git branch
    let git_branch = get_git_branch().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SG_ROUTER_GIT_BRANCH={}", git_branch);

    // Try to get Git commit hash
    let git_commit = get_git_commit().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SG_ROUTER_GIT_COMMIT={}", git_commit);

    // Try to get Git status (clean/dirty)
    let git_status = get_git_status().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SG_ROUTER_GIT_STATUS={}", git_status);

    // Get Rustc version
    let rustc_version = get_rustc_version().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SG_ROUTER_RUSTC_VERSION={}", rustc_version);

    // Get Cargo version
    let cargo_version = get_cargo_version().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SG_ROUTER_CARGO_VERSION={}", cargo_version);

    // Get target triple (platform)
    let target_triple = std::env::var("TARGET").unwrap_or_else(|_| {
        // Try to get from rustc if not set
        get_target_from_rustc().unwrap_or_else(|| "unknown".to_string())
    });
    println!("cargo:rustc-env=SG_ROUTER_TARGET_TRIPLE={}", target_triple);

    // Get build mode (debug/release)
    let build_mode = if std::env::var("PROFILE").unwrap_or_default() == "release" {
        "release"
    } else {
        "debug"
    };
    println!("cargo:rustc-env=SG_ROUTER_BUILD_MODE={}", build_mode);

    Ok(())
}

fn read_version_from_pyproject(field: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("pyproject.toml")?;

    // Simple TOML parsing for specified field
    for line in content.lines() {
        let line = line.trim();
        let prefix = format!("{} = ", field);
        if line.starts_with(&prefix) {
            let value = line
                .strip_prefix(&prefix)
                .ok_or(format!("Failed to parse {} line", field))?
                .trim();
            // Remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'');
            return Ok(value.to_string());
        }
    }

    Err(format!("{} not found in pyproject.toml", field).into())
}

fn get_git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn get_git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn get_git_status() -> Option<String> {
    // Check if there are uncommitted changes
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .ok()?;

    if output.status.success() {
        if output.stdout.is_empty() {
            Some("clean".to_string())
        } else {
            Some("dirty".to_string())
        }
    } else {
        None
    }
}

fn get_rustc_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn get_cargo_version() -> Option<String> {
    let output = Command::new("cargo").arg("--version").output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn get_target_from_rustc() -> Option<String> {
    let output = Command::new("rustc").args(&["-vV"]).output().ok()?;

    if output.status.success() {
        let output_str = String::from_utf8(output.stdout).ok()?;
        for line in output_str.lines() {
            if line.starts_with("host: ") {
                if let Some(host) = line.strip_prefix("host: ") {
                    return Some(host.trim().to_string());
                }
            }
        }
    }
    None
}
