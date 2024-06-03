use rand::Rng;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;
use std::{env, fs};

static BUILD: Once = Once::new();

pub fn setup() {
    BUILD.call_once(|| {
        assert!(Command::new("cargo")
            .arg("build")
            .output()
            .expect("failed to build extension")
            .status
            .success());
    });
}

pub fn write_test_file(script_name: &str, code: &str) -> PathBuf {
    let script_filename = env::current_dir().unwrap().join("tests/temp").join(script_name);
    fs::write(script_filename.clone(), code).unwrap();
    script_filename
}

pub fn php_request(code: &str) -> String {
    let rand_name = rand::thread_rng().gen_range(1..99999999).to_string() + ".php";
    let script_name = rand_name.as_str();
    let script_filename = write_test_file(&script_name, code);

    let res = php_request_file(script_filename.to_str().unwrap());
    fs::remove_file(script_filename).unwrap();

    res
}

pub fn php_request_file(script_filename: &str) -> String {
    let output = Command::new("php")
        // .arg(format!(
        //     "-dextension={}/target/debug/lib{}.{}",
        //     env::current_dir().unwrap().to_str().unwrap(),
        //     env!("CARGO_PKG_NAME"),
        //     std::env::consts::DLL_EXTENSION,
        // ))
        .arg(script_filename)
        .output()
        .expect("failed to execute PHP script");

    if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        panic!(
            "PHP script execution failed: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }
}