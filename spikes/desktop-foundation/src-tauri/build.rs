fn main() {
    let rgba = [101, 88, 211, 255].repeat(32 * 32);
    let png_path = std::path::Path::new("icons/icon.png");
    if !png_path.exists() {
        std::fs::create_dir_all("icons").unwrap();
        let file = std::fs::File::create(png_path).unwrap();
        let mut encoder = png::Encoder::new(file, 32, 32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&rgba).unwrap();
    }
    let ico_path = std::path::Path::new("icons/icon.ico");
    if !ico_path.exists() {
        let image = ico::IconImage::from_rgba_data(32, 32, rgba);
        let mut icon = ico::IconDir::new(ico::ResourceType::Icon);
        icon.add_entry(ico::IconDirEntry::encode(&image).unwrap());
        icon.write(std::fs::File::create(ico_path).unwrap()).unwrap();
    }
    tauri_build::build()
}
