use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let current_dir = env::current_dir().unwrap();

    std::thread::scope(|s| {
        let widget_api_dir = current_dir.join("widget-api");
        s.spawn(move || npm_run_build(widget_api_dir));
        println!("cargo::rerun-if-changed=widget-api/src");
        println!("cargo::rerun-if-changed=widget-api/package.json");

        let widgets_dir = current_dir.join("widgets");
        std::fs::read_dir(widgets_dir)
            .unwrap()
            .flatten()
            .filter(|entry| entry.metadata().unwrap().is_dir())
            .for_each(|dir| {
                let path = dir.path();
                println!("cargo::rerun-if-changed={}/src", path.display());
                println!("cargo::rerun-if-changed={}/package.json", path.display());
                s.spawn(move || npm_run_build(path));
            })
    });
}

fn npm_run_build(path: PathBuf) {
    let npm_install_status = Command::new("npm")
        .current_dir(path.clone())
        .arg("install")
        .arg("--prefer-offline")
        .arg("--progress=false")
        .status()
        .expect("Failed to execute 'npm install'");
    if !npm_install_status.success() {
        panic!("'npm install' exited with: {npm_install_status}");
    }

    let npm_build_status = Command::new("npm")
        .current_dir(path)
        .arg("run")
        .arg("build")
        .status()
        .expect("Failed to execute 'npm run build'");
    if !npm_build_status.success() {
        panic!("'npm run build' exited with: {npm_build_status}");
    }
}
