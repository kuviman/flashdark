extern crate winres;

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("static/assets/icon.ico");
        res.compile().unwrap();
    }
}
