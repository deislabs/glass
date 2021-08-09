wacm::export!("glass_runtime");

struct GlassRuntime {}

impl glass_runtime::GlassRuntime for GlassRuntime {
    fn handler(req: glass_runtime::Request) -> glass_runtime::Response {
        println!("request: {:?}, {:?}, {:?}", req.0, req.1, req.2);

        (200, None, None)
    }
}

// Workaround for the linker, who believes this should be
// a command / executable component.
#[no_mangle]
pub fn _start() {
    // let mut buffer = String::new();
    // std::io::stdin().read_to_string(&mut buffer).unwrap();
    // print!("{}", markdown::render(&buffer));
}
