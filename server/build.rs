use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let current_dir = env::current_dir().unwrap();

    // widget-lib
    let widget_lib_dir = current_dir.join("widget-lib");
    npm_run_build(widget_lib_dir);
    // widget-api
    let widget_api_dir = current_dir.join("widget-api");
    let widget_api_handle = std::thread::spawn(move || npm_run_build(widget_api_dir));
    widget_api_handle.join().unwrap();

    let widgets_dir = current_dir.join("widgets");
    std::fs::read_dir(widgets_dir)
        .unwrap()
        .flatten()
        .filter(|entry| entry.metadata().unwrap().is_dir())
        .map(|dir| {
            let path = dir.path();
            println!("cargo:rerun-if-changed={}/src", path.display());
            println!("cargo:rerun-if-changed={}/package.json", path.display());
            std::thread::spawn(move || npm_run_build(path))
        })
        // .chain(std::iter::once(widget_api_handle))
        .for_each(|h| h.join().expect("failed joining build command"));

    // Tell Cargo to rerun this script if any these changes
    println!("cargo:rerun-if-changed=server/widget-lib/src");
    println!("cargo:rerun-if-changed=server/widget-lib/package.json");
    println!("cargo:rerun-if-changed=server/widget-api/src");
    println!("cargo:rerun-if-changed=server/widget-api/package.json");
}

fn npm_run_build(path: PathBuf) {
    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&path).unwrap();

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
}
