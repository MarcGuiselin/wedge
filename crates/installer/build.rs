fn main() {
    // Reload changes from resource files
    println!("cargo:rerun-if-changed=src/ack_dialog.rc");
    println!("cargo:rerun-if-changed=src/install_dialog.rc");
    println!("cargo:rerun-if-changed=src/embed.rc");
    println!("cargo:rerun-if-changed=../../LICENSE");

    // Compile
    wedge_lib::build::compile_using_template_resource_file(
        "Wedge Installer",
        "wedge.installer",
        vec!["src/ack_dialog.rc", "src/install_dialog.rc", "src/embed.rc"],
        vec![
            if cfg!(debug_assertions) {
                "../../target/debug/uninstaller.exe"
            } else {
                "../../target/release/uninstaller.exe"
            },
            if cfg!(debug_assertions) {
                "../../target/debug/wedge.exe"
            } else {
                "../../target/release/wedge.exe"
            },
            "../../LICENSE",
        ],
    );
}
