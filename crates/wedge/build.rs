use wedge_lib::build::*;

fn main() {
    compile_using_template_resource_file(
        "Wedge",
        "wedge.app",
        vec![],
        vec![],
        ExecutionLevel::AsInvoker,
    );
}
