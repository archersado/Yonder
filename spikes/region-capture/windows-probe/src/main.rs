#[path = "../../../../apps/desktop/src/region_capture_windows.rs"]
mod region_capture_windows;

use serde_json::json;

fn main() {
    let values = std::env::args().skip(1).map(|value| value.parse::<i32>()).collect::<Result<Vec<_>, _>>();
    let Ok(values) = values else { finish(json!({"passed": false, "error": "invalid-arguments"}), 2); };
    if values.len() != 4 || values[2] <= 0 || values[3] <= 0 {
        finish(json!({"passed": false, "error": "invalid-arguments"}), 2);
    }
    let (x, y, width, height) = (values[0], values[1], values[2] as u32, values[3] as u32);
    match region_capture_windows::capture_region(x, y, width, height) {
        Ok(capture) => {
            let checks = color_checks(&capture.rgba, capture.width, capture.height);
            let capture_size = (capture.width, capture.height);
            drop(capture);
            finish(json!({
                "passed": checks.iter().all(|value| *value),
                "request": {"x": x, "y": y, "width": width, "height": height},
                "capture": {"pixels_width": capture_size.0, "pixels_height": capture_size.1},
                "color_checks": checks,
                "negative_origin": x < 0 || y < 0,
                "pixels_released_after_check": true,
                "screenshot_persisted": false
            }), if checks.iter().all(|value| *value) { 0 } else { 2 });
        }
        Err(error) => finish(json!({
            "passed": false,
            "error": error.code(),
            "request": {"x": x, "y": y, "width": width, "height": height},
            "screenshot_persisted": false
        }), 2),
    }
}

fn color_checks(rgba: &[u8], width: u32, height: u32) -> [bool; 4] {
    let samples = [(width / 4, height / 4), (width * 3 / 4, height / 4), (width / 4, height * 3 / 4), (width * 3 / 4, height * 3 / 4)];
    let pixels = samples.map(|(x, y)| {
        let offset = ((y * width + x) * 4) as usize;
        [rgba[offset], rgba[offset + 1], rgba[offset + 2]]
    });
    [
        pixels[0][0] > 200 && pixels[0][1] < 80 && pixels[0][2] < 80,
        pixels[1][1] > 200 && pixels[1][0] < 80 && pixels[1][2] < 80,
        pixels[2][2] > 200 && pixels[2][0] < 80 && pixels[2][1] < 80,
        pixels[3][0] > 200 && pixels[3][1] > 200 && pixels[3][2] < 80,
    ]
}

fn finish(value: serde_json::Value, code: i32) -> ! {
    println!("{}", serde_json::to_string_pretty(&value).expect("结构化证据"));
    std::process::exit(code);
}
