use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let current_dir = env::current_dir().unwrap();

    std::thread::scope(|s| {
        let api_dir = current_dir.join("..").join("widget-api");
        println!("cargo::rerun-if-changed={}/src/", api_dir.display());
        println!("cargo::rerun-if-changed={}/package.json", api_dir.display());
        s.spawn(move || npm_run_build(api_dir));

        let widgets_dir = current_dir.join("..").join("widgets");
        std::fs::read_dir(widgets_dir)
            .unwrap()
            .flatten()
            .filter(|entry| entry.metadata().unwrap().is_dir())
            .for_each(|dir| {
                let path = dir.path();
                let is_npm_project = std::fs::read_dir(path.clone())
                    .unwrap()
                    .any(|f| f.unwrap().file_name().as_os_str() == "package.json");
                if !is_npm_project {
                    return;
                }

                println!("cargo::rerun-if-changed={}/src/", path.display());
                println!("cargo::rerun-if-changed={}/package.json", path.display());
                s.spawn(move || npm_run_build(path));
            })
    });
    println!("cargo:rerun-if-changed=../migrations/");
}

fn npm_run_build(path: impl Into<PathBuf>) {
    let path = path.into();
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
