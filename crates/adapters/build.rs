use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/work_focus_macos.c");
    if !env::var("TARGET").unwrap_or_default().contains("apple-darwin") { return; }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let object = out.join("work_focus_macos.o");
    let library = out.join("libyonda_work_focus.a");
    assert!(Command::new("clang").args(["-c","src/work_focus_macos.c","-o"]).arg(&object).args(["-O2","-Wall","-Wextra"]).status().is_ok_and(|status| status.success()));
    assert!(Command::new("ar").arg("crus").arg(&library).arg(&object).status().is_ok_and(|status| status.success()));
    println!("cargo:rustc-link-search=native={}",out.display());
    println!("cargo:rustc-link-lib=static=yonda_work_focus");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
}
