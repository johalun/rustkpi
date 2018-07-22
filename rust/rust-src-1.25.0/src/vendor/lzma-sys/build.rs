extern crate cc;
extern crate filetime;
extern crate pkg_config;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

use filetime::FileTime;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} return the error {}", stringify!($e), e),
    })
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo:rerun-if-env-changed=LZMA_API_STATIC");
    let want_static = env::var("LZMA_API_STATIC").is_ok();
    if !want_static && pkg_config::probe_library("liblzma").is_ok() {
        return;
    }

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    println!("cargo:rerun-if-changed=xz-5.2.3/configure");
    let features = env::var("CARGO_CFG_TARGET_FEATURE")
                        .unwrap_or(String::new());
    if target.contains("msvc") {
        println!("cargo:rustc-link-lib=static=liblzma");
        let build = dst.join("build");
        let _ = fs::remove_dir_all(&build);
        let _ = fs::remove_dir_all(dst.join("lib"));
        let _ = fs::remove_dir_all(dst.join("include"));
        cp_r(Path::new("xz-5.2.3"), &build);

        let profile = if features.contains("crt-static") {
            "ReleaseMT"
        } else {
            "Release"
        };

        let mut build_succeeded = false;
        for platform_toolset in &["v141", "v140", "v120", "v110"] {
            let mut msbuild = cc::windows_registry::find(&target, "msbuild")
                    .expect("needs msbuild installed");
            if try_run(msbuild.current_dir(build.join("windows"))
                    .arg("liblzma.vcxproj")
                    .arg(&format!("/p:Configuration={}", profile))
                    .arg(&format!("/p:PlatformToolset={}", platform_toolset))) {
                build_succeeded = true;
                break;
            }
        }
        assert!(build_succeeded);

        t!(fs::create_dir(dst.join("lib")));
        t!(fs::create_dir(dst.join("include")));
        let platform = if target.contains("x86_64") {"X64"} else {"Win32"};
        t!(fs::copy(build.join("windows")
                         .join(profile)
                         .join(platform)
                         .join("liblzma/liblzma.lib"),
                    dst.join("lib/liblzma.lib")));
        t!(fs::copy(build.join("src/liblzma/api/lzma.h"),
                    dst.join("include/lzma.h")));
        cp_r(&build.join("src/liblzma/api/lzma"), &dst.join("include/lzma"));
    } else {
        // Looks like xz-5.2.2's build system is super sensitive to mtimes, so
        // if we just blindly use what's on the filesystem it's likely to try to
        // run tons of automake junk or modify files in the build directory,
        // neither of which we want.
        //
        // Work around this by just touching every file to the same time.
        let src = dst.join("src");
        drop(fs::remove_dir_all(&src));
        cp_r(Path::new("xz-5.2.3"), &src);
        let meta = t!(src.join("configure").metadata());
        let now = FileTime::from_last_modification_time(&meta);
        set_all_mtime(&src, &now);

        println!("cargo:rustc-link-lib=static=lzma");
        let cfg = cc::Build::new();
        let compiler = cfg.get_compiler();

        let _ = fs::create_dir(&dst.join("build"));

        let mut cmd = Command::new("sh");
        let mut cflags = OsString::new();
        for arg in compiler.args() {
            cflags.push(arg);
            cflags.push(" ");
        }
        cmd.env("CC", compiler.path())
           .env("CFLAGS", cflags)
           .current_dir(&dst.join("build"))
           .arg(sanitize_sh(&src.join("configure")));
        cmd.arg(format!("--prefix={}", sanitize_sh(&dst)));
        cmd.arg("--disable-doc");
        cmd.arg("--disable-lzma-links");
        cmd.arg("--disable-lzmainfo");
        cmd.arg("--disable-lzmadec");
        cmd.arg("--disable-xz");
        cmd.arg("--disable-xzdec");
        cmd.arg("--disable-scripts");
        cmd.arg("--disable-shared");
        cmd.arg("--disable-nls");
        cmd.arg("--disable-rpath");

        if target.contains("windows") {
            cmd.arg("--enable-threads=win95");
        } else {
            cmd.arg("--enable-threads=yes");
        }

        if target != host &&
           (!target.contains("windows") || !host.contains("windows")) {
            // NOTE GNU terminology
            // BUILD = machine where we are (cross) compiling curl
            // HOST = machine where the compiled curl will be used
            // TARGET = only relevant when compiling compilers
            if target.contains("windows") {
                // curl's configure can't parse `-windows-` triples when used
                // as `--host`s. In those cases we use this combination of
                // `host` and `target` that appears to do the right thing.
                cmd.arg(format!("--host={}", host));
                cmd.arg(format!("--target={}", target));
            } else {
                cmd.arg(format!("--build={}", host));
                cmd.arg(format!("--host={}", target));
            }
        }

        run(&mut cmd);
        run(make(&host)
                    .arg(&format!("-j{}", env::var("NUM_JOBS").unwrap()))
                    .current_dir(&dst.join("build")));
        // Unset DESTDIR or liblzma.a ends up in it and cargo can't find it
        // MAKEFLAGS may also contain DESTDIR (from 'make install DESTDIR=...')
        // so we should get rid of that as well
        env::remove_var("DESTDIR");
        env::remove_var("MAKEFLAGS");
        run(make(&host)
                    .arg("install")
                    .current_dir(&dst.join("build/src/liblzma")));
    }
}

fn make(host: &String) -> Command {
    let mut cmd = if host.contains("bitrig") || host.contains("dragonfly") ||
        host.contains("freebsd") || host.contains("netbsd") ||
        host.contains("openbsd") || host.contains("solaris") {

        Command::new("gmake")
    } else {
        Command::new("make")
    };

    // We're using the MSYS make which doesn't work with the mingw32-make-style
    // MAKEFLAGS, so remove that from the env if present.
    if cfg!(windows) {
        cmd.env_remove("MAKEFLAGS").env_remove("MFLAGS");
    }

    return cmd
}

fn try_run(cmd: &mut Command) -> bool {
    println!("running: {:?}", cmd);
    t!(cmd.status()).success()
}

fn run(cmd: &mut Command) {
    assert!(try_run(cmd));
}

fn cp_r(src: &Path, dst: &Path) {
    t!(fs::create_dir(dst));
    for e in t!(src.read_dir()).map(|e| t!(e)) {
        let src = e.path();
        let dst = dst.join(e.file_name());
        if t!(e.file_type()).is_dir() {
            cp_r(&src, &dst);
        } else {
            t!(fs::copy(&src, &dst));
        }
    }
}

fn set_all_mtime(path: &Path, mtime: &FileTime) {
    for e in t!(path.read_dir()).map(|e| t!(e)) {
        let path = e.path();
        if t!(e.file_type()).is_dir() {
            set_all_mtime(&path, mtime);
        } else {
            t!(filetime::set_file_times(&path, *mtime, *mtime));
        }
    }
}

fn sanitize_sh(path: &Path) -> String {
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None
        }
        if ch.next() != Some('/') {
            return None
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}
