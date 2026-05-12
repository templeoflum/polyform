#[cfg(target_os = "windows")]
fn main() {
    let sdk_dir = "spout-sdk";

    cc::Build::new()
        .cpp(true)
        .files([
            format!("{sdk_dir}/spout_bridge.cpp"),
            format!("{sdk_dir}/SpoutDX.cpp"),
            format!("{sdk_dir}/SpoutDirectX.cpp"),
            format!("{sdk_dir}/SpoutCopy.cpp"),
            format!("{sdk_dir}/SpoutFrameCount.cpp"),
            format!("{sdk_dir}/SpoutSenderNames.cpp"),
            format!("{sdk_dir}/SpoutSharedMemory.cpp"),
            format!("{sdk_dir}/SpoutUtils.cpp"),
        ])
        .include(sdk_dir)
        .define("SPOUT_NO_TASKDIALOG", None)
        .flag("/EHsc")
        .flag("/std:c++17")
        .compile("spout_bridge");

    // System libraries required by SpoutDX
    println!("cargo:rustc-link-lib=d3d11");
    println!("cargo:rustc-link-lib=dxgi");
    println!("cargo:rustc-link-lib=shlwapi");
    println!("cargo:rustc-link-lib=version");
    println!("cargo:rustc-link-lib=opengl32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=winmm");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Spout is Windows-only; nothing to build on other platforms.
}
