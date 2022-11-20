use wedge_lib::build::*;

fn main() {
    compile_using_template_resource_file(
        "Wedge Uninstaller",
        "wedge.uninstaller",
        vec![],
        vec![],
        ExecutionLevel::RequireAdministrator,
    );
}
