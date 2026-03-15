use eframe;

mod window;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "stats",
        native_options,
        Box::new(|cc| Ok(Box::new(window::MyApp::new(cc)))),
    );
}


