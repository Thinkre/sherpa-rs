// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Setup library paths early (before any libraries are loaded)
    setup_sherpa_library_paths_early();

    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/dri").exists()
            && std::env::var("WAYLAND_DISPLAY").is_err()
            && std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11"
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    kevoiceinput_app_lib::run()
}

/// Setup library paths very early, before any dynamic libraries are loaded
/// This is critical on macOS where DYLD_LIBRARY_PATH must be set before process starts
fn setup_sherpa_library_paths_early() {
    // Check if SHERPA_LIB_PATH is already set
    if std::env::var("SHERPA_LIB_PATH").is_ok() {
        eprintln!("[INFO] SHERPA_LIB_PATH already set, skipping auto-detection");
        return;
    }

    // Check for project's own vendored libraries first (macOS arm64)
    let arch = std::env::consts::ARCH;
    let platform_suffix = match (std::env::consts::OS, arch) {
        ("macos", "aarch64") => Some("macos-arm64"),
        ("macos", "x86_64") => Some("macos-x64"),
        ("linux", "aarch64") => Some("linux-arm64"),
        ("linux", "x86_64") => Some("linux-x64"),
        ("windows", "x86_64") => Some("windows-x64"),
        _ => None,
    };

    if let Some(suffix) = platform_suffix {
        let vendor_lib_path = format!("vendor/libs/{}", suffix);
        let vendor_abs_path = std::path::Path::new(&vendor_lib_path)
            .canonicalize()
            .unwrap_or(std::path::PathBuf::from(&vendor_lib_path));

        if vendor_abs_path.exists() {
            // Check if the vendor directory contains dynamic libraries
            let has_dylibs = std::fs::read_dir(&vendor_abs_path)
                .map(|mut entries| {
                    entries.any(|e| {
                        e.ok()
                            .and_then(|e| {
                                e.file_name().to_str().map(|s| {
                                    s.ends_with(".dylib")
                                        || s.ends_with(".so")
                                        || s.ends_with(".dll")
                                })
                            })
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_dylibs {
                eprintln!(
                    "[INFO] Found vendored libraries in project at: {}",
                    vendor_abs_path.display()
                );
                std::env::set_var("SHERPA_LIB_PATH", &vendor_abs_path);

                // Set DYLD_LIBRARY_PATH for macOS (must be done before libraries load)
                #[cfg(target_os = "macos")]
                {
                    let current_dyld = std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
                    let new_dyld = if current_dyld.is_empty() {
                        vendor_abs_path.to_string_lossy().into_owned()
                    } else {
                        format!("{}:{}", vendor_abs_path.to_string_lossy(), current_dyld)
                    };
                    std::env::set_var("DYLD_LIBRARY_PATH", &new_dyld);
                    eprintln!("[INFO] Set DYLD_LIBRARY_PATH to: {}", new_dyld);
                }

                return;
            }
        }
    }

    // Check for local sherpa-onnx build as fallback
    let local_sherpa_path = "/Users/thinkre/Desktop/open/sherpa-onnx/build";
    if !std::path::Path::new(local_sherpa_path).exists() {
        eprintln!("[INFO] Local sherpa-onnx build not found, using default (download-binaries or project vendor libs)");
        return;
    }

    // Check if local build has dynamic libraries (required by sherpa-rs-sys)
    let lib_dir = format!("{}/lib", local_sherpa_path);
    let has_dylib = std::path::Path::new(&lib_dir).exists()
        && std::fs::read_dir(&lib_dir)
            .map(|mut entries| {
                entries.any(|e| {
                    e.ok()
                        .and_then(|e| e.file_name().to_str().map(|s| s.ends_with(".dylib")))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

    if !has_dylib {
        eprintln!("[WARN] Local sherpa-onnx build found but only static libraries (.a) detected.");
        eprintln!("[WARN] sherpa-rs-sys requires dynamic libraries (.dylib).");
        eprintln!("[WARN] Either:");
        eprintln!("[WARN]   1. Rebuild sherpa-onnx with BUILD_SHARED_LIBS=ON");
        eprintln!("[WARN]   2. Or unset SHERPA_LIB_PATH to use official pre-built libraries");
        eprintln!("[WARN] Skipping SHERPA_LIB_PATH setup - will use download-binaries");
        return;
    }

    eprintln!(
        "[INFO] Found local sherpa-onnx build with dynamic libraries at: {}",
        local_sherpa_path
    );
    std::env::set_var("SHERPA_LIB_PATH", local_sherpa_path);

    // Set DYLD_LIBRARY_PATH for macOS (must be done before libraries load)
    #[cfg(target_os = "macos")]
    {
        if std::path::Path::new(&lib_dir).exists() {
            let current_dyld = std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
            let new_dyld = if current_dyld.is_empty() {
                lib_dir.clone()
            } else {
                format!("{}:{}", lib_dir, current_dyld)
            };
            std::env::set_var("DYLD_LIBRARY_PATH", &new_dyld);
            eprintln!("[INFO] Set DYLD_LIBRARY_PATH to: {}", new_dyld);
        }

        // Also check for ONNX Runtime in sherpa-rs cache
        if let Ok(home) = std::env::var("HOME") {
            let cache_base = format!("{}/Library/Caches/sherpa-rs", home);
            if let Ok(entries) = std::fs::read_dir(&cache_base) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let lib_path = entry
                            .path()
                            .join("sherpa-onnx-v1.12.9-osx-universal2-shared/lib");
                        if lib_path.exists() {
                            let current_dyld =
                                std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
                            let lib_str = lib_path.to_string_lossy().to_string();
                            if !current_dyld.contains(&lib_str) {
                                let new_dyld = if current_dyld.is_empty() {
                                    lib_str.clone()
                                } else {
                                    format!("{}:{}", lib_str, current_dyld)
                                };
                                std::env::set_var("DYLD_LIBRARY_PATH", &new_dyld);
                                eprintln!(
                                    "[INFO] Added ONNX Runtime to DYLD_LIBRARY_PATH: {}",
                                    lib_str
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}
