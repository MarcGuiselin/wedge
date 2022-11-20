use cargo_metadata::*;
use fs_extra::dir::{copy, CopyOptions};
use handlebars::Handlebars;
use serde_json::json;
use std::{
    env::{args, var_os},
    fs::{read_to_string, write},
    path::Path,
};

/// Load environment variables from root
macro_rules! env_var {
    ($i:expr) => {
        &format!("{}", var_os($i).unwrap().to_str().unwrap())
    };
}

/// (Probably) overcomplicated way to get license from cargo pkg.
/// Copied from MetadataCommand documentation.
fn get_package_details() -> (String, String) {
    let mut args = args().skip_while(|val| !val.starts_with("--manifest-path"));
    let mut cmd = MetadataCommand::new();
    match args.next() {
        Some(ref p) if p == "--manifest-path" => {
            cmd.manifest_path(args.next().unwrap());
        }
        Some(p) => {
            cmd.manifest_path(p.trim_start_matches("--manifest-path="));
        }
        None => {}
    };
    let metadata = cmd.exec().unwrap();

    // Get license from metadata
    let mut ret = None;
    if let Some(resolve) = &metadata.resolve {
        if let Some(root) = &resolve.root {
            if let Some(license) = &metadata[&root].license {
                ret = Some(String::from(license));
            }
        }
    }

    // Get workspace root
    let ret2 = String::from(metadata.workspace_root.as_str());
    (ret.unwrap(), ret2)
}

/// Compile final binaries using template resource files
pub fn compile_using_template_resource_file(
    name: &str,
    dotname: &str,
    resource_appends: Vec<&str>,
    additional_resources: Vec<&str>,
) {
    let out_dir = env_var!("OUT_DIR");
    let out_dir = Path::new(out_dir);
    let (license, workspace_root) = get_package_details();

    // Always path to resource folder
    // let resources_dir = env_var!("CARGO_MANIFEST_DIR");
    let resources_dir = Path::new(&workspace_root).join("resources");

    // Reload changes from template resource + template manifest file
    println!(
        "cargo:rerun-if-changed={}",
        resources_dir.join("template.rc").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        resources_dir.join("template.manifest").to_str().unwrap()
    );

    // Copy resources to out_dir
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy(resources_dir.to_str().unwrap(), out_dir, &options)
        .expect("Something went wrong copying resources to out_dir");

    // Copy additional resources
    for file in additional_resources {
        let file = Path::new(file);
        std::fs::copy(
            file,
            out_dir.join("resources").join(file.file_name().unwrap()),
        )
        .expect("Could not copy extra resource to out_dir");
    }

    // Paths all in OUT_DIR
    let rc_path = out_dir.join(r"resources\template.rc");
    let manifest_path = out_dir.join(r"resources\template.manifest");

    // Load templates
    let rc_template = read_to_string(rc_path.to_str().unwrap())
        .expect("Something went wrong reading template.rc");
    let manifest_template = read_to_string(manifest_path.to_str().unwrap())
        .expect("Something went wrong reading template.manifest");

    // Template settings
    let version = [
        env_var!("CARGO_PKG_VERSION_MAJOR"),
        env_var!("CARGO_PKG_VERSION_MINOR"),
        env_var!("CARGO_PKG_VERSION_PATCH"),
        "0",
    ];
    let settings = &json!({
        "name": name, //"Wedge Installer",
        "dotname": dotname, //"MarcGuiselin.Wedge.Installer",
        "description": env_var!("CARGO_PKG_DESCRIPTION"),
        "copyright": license,
        "version": version[0..3].join("."), // 1.2.3
        "four_digit_version": version.join("."), // 1.2.3.0
        "four_digit_comma_separated_version": version.join(", "), // 1, 2, 3, 0
    });

    // Render templates
    let reg = Handlebars::new();
    let mut rc_text = reg
        .render_template(&rc_template, &settings)
        .expect("Something went wrong redering template from template.rc");
    let manifest_text = reg
        .render_template(&manifest_template, &settings)
        .expect("Something went wrong redering template from template.manifest");

    // Append contents of other resource files
    for file in resource_appends {
        rc_text.push_str(&format!(
            "\n\n// ========================== \n// {} \n// ========================== \n\n",
            &file
        ));
        rc_text.push_str(&read_to_string(&file).unwrap_or_else(|_| {
            panic!(
                "Something went wrong reading append resource file: {}",
                &file
            )
        }));
    }

    // Write rendered text to files
    write(&rc_path, &rc_text).unwrap();
    write(&manifest_path, &manifest_text).unwrap();

    // Compile and link checksums.rc
    embed_resource::compile(rc_path.to_str().unwrap());
}
