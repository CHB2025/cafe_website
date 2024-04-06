use std::{env, path::Path, process::Command};

// generated by `sqlx migrate build-script`
fn main() {
    // trigger recompilation when a new migration is added/css changes
    println!("cargo:rerun-if-changed=migrations,templates");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");

    let mut cmd = Command::new("tailwindcss");
    cmd.arg("-i")
        .arg("./index.css")
        .arg("-o")
        .arg(dest_path.to_str().unwrap());
    if env::var_os("OPT_LEVEL").is_some_and(|l| l == "3") {
        cmd.arg("-m");
    }
    cmd.output().expect("Tailwind failed to execute");
}
