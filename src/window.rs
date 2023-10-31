use minifb::{/* Key, */ Scale, ScaleMode, Window, WindowOptions};

pub fn create_window() -> minifb::Window {
    let mut window = Window::new(
        "Fractal - ESC to exit",
        crate::vm::DISPLAY_WIDTH,
        crate::vm::DISPLAY_HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X16,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    window.set_background_color(20, 20, 30);

    window
}

// pub fn update_window_with_buffer(w: &mut minifb::Window, buf: &[u32]) {
//     w.update_with_buffer(&buf, crate::vm::DISPLAY_WIDTH, crate::vm::DISPLAY_HEIGHT)
//         .unwrap();
// }
