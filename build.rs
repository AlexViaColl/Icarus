use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-search=.");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=compile.sh");
    println!("cargo:rerun-if-changed=assets/shaders/");

    // Download flappy bird assets
    let path = "assets/textures/flappy";
    let url = "https://raw.githubusercontent.com/samuelcust/flappy-bird-assets/sprites";
    download_if_not_present(format!("{}/background-day.png", path), format!("{}/background-day.png", url));
    download_if_not_present(format!("{}/base.png", path), format!("{}/base.png", url));
    download_if_not_present(format!("{}/bluebird-downflap.png", path), format!("{}/bluebird-downflap.png", url));
    download_if_not_present(format!("{}/bluebird-midflap.png", path), format!("{}/bluebird-midflap.png", url));
    download_if_not_present(format!("{}/bluebird-upflap.png", path), format!("{}/bluebird-upflap.png", url));
    download_if_not_present(format!("{}/pipe-green.png", path), format!("{}/pipe-green.png", url));

    // Download stb_image
    download_if_not_present("stb_image.h", "https://raw.githubusercontent.com/nothings/stb/master/stb_image.h");

    if !Path::new("glslc").exists() {
        Command::new("/usr/bin/wget")
            .arg("https://raw.githubusercontent.com/AlexViaColl/Icarus_deps/main/glslc.tar.gz")
            .status()
            .unwrap();

        Command::new("/usr/bin/tar").arg("-xzvf").arg("glslc.tar.gz").status().unwrap();
        std::fs::set_permissions("glslc", std::fs::Permissions::from_mode(0o770)).unwrap();
    }

    let output = Command::new("/bin/sh").arg("compile.sh").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "stderr: {}\nstdout: {}", stderr, stdout);
}

fn download_if_not_present<P: AsRef<OsStr>>(path: P, url: P) {
    if !Path::new(&path).exists() {
        Command::new("/usr/bin/wget").arg("-O").arg(path).arg(url).status().unwrap();
    }
}
