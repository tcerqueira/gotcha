use std::env;
use std::process::Command;

fn main() {
    let current_dir = env::current_dir().unwrap();
    let widget_dir = current_dir.join("widget-api");
    // Change dir to /widget/
    env::set_current_dir(&widget_dir).unwrap();

    let npm_install_status = Command::new("npm")
        .arg("install")
        .status()
        .expect("Failed to execute 'npm install'");
    if !npm_install_status.success() {
        panic!("'npm install' exited with: {npm_install_status}");
    }

    let npm_build_status = Command::new("npm")
        .arg("run")
        .arg("build")
        .status()
        .expect("Failed to execute 'npm run build'");
    if !npm_build_status.success() {
        panic!("'npm run build' exited with: {npm_build_status}");
    }

    // Change back to the original directory
    env::set_current_dir(current_dir).unwrap();
    // Tell Cargo to rerun this script if any these changes
    println!("cargo:rerun-if-changed=server/widget/src");
    println!("cargo:rerun-if-changed=server/widget/package.json");
}
