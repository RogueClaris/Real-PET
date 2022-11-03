use fs_extra;
use std::fs;
use std::process::Command;

const CLIENT_NAME: &str = "real_pet_server";

fn main() {
    let build_output = Command::new("cargo")
        .args(["build", "-p", CLIENT_NAME, "--release"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .unwrap();

    if !build_output.status.success() {
        // stdout + stderr are shared, no need to display anything
        return;
    }

    // areas
    fs_extra::dir::create_all("dist/server", true).unwrap();
    fs_extra::dir::copy(
        "server/areas",
        "dist/server",
        &fs_extra::dir::CopyOptions::default(),
    )
    .unwrap();
    fs_extra::dir::copy(
        "server/assets",
        "dist/server",
        &fs_extra::dir::CopyOptions::default(),
    )
    .unwrap();

    // scripts
    fs_extra::dir::create_all("dist/server/scripts", true).unwrap();

    fs_extra::dir::copy(
        "server/scripts/libs",
        "dist/server/scripts",
        &fs_extra::dir::CopyOptions::default(),
    )
    .unwrap();

    // windows exe
    let _ = fs::copy(
        format!("target/release/{CLIENT_NAME}.exe"),
        format!("dist/server/{CLIENT_NAME}.exe"),
    );

    // linux exe
    let _ = fs::copy(
        format!("target/release/{CLIENT_NAME}"),
        format!("dist/server/{CLIENT_NAME}"),
    );
}