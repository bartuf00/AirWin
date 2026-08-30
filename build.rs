// Build script: embeds the application icon into the Windows executable
// resources (Explorer/taskbar/alt-tab icon). The window title-bar icon is
// loaded separately at runtime from the embedded PNG in `ui::window_icon()`.

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let png_path = PathBuf::from(&manifest_dir).join("assets").join("icon.png");
    println!("cargo:rerun-if-changed={}", png_path.display());

    // Decode the PNG (RGBA8) once; used for both the .ico and the runtime icon.
    let png = fs::read(&png_path).expect("assets/icon.png missing");
    let img = image::load_from_memory(&png)
        .expect("assets/icon.png is not a valid image")
        .to_rgba8();
    let (w, h) = img.dimensions();
    assert_eq!(w, h, "icon must be square");

    // Write a minimal .ico next to the binary for winresource.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ico_path = out_dir.join("icon.ico");
    write_ico(&img, &ico_path);

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.set("ProductName", "AirWin");
        res.set("FileDescription", "AirWin - Apple sharing for Windows");
        if let Err(e) = res.compile() {
            // Don't fail the build if the resource compiler is unavailable
            // (e.g. cross-compiling without windres); the runtime icon still works.
            println!("cargo:warning=winresource failed: {}", e);
        }
    }
}

fn write_ico(img: &image::RgbaImage, path: &PathBuf) {
    let (w, h) = img.dimensions();
    // PNG-compressed payload is allowed in .ico since Vista.
    let mut png_buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png_buf, image::ImageFormat::Png).unwrap();
    let png_bytes = png_buf.into_inner();

    let mut ico = Vec::new();
    // ICONDIR
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type = icon
    ico.extend_from_slice(&1u16.to_le_bytes()); // count
    // ICONDIRENTRY
    ico.push(if w >= 256 { 0 } else { w as u8 });
    ico.push(if h >= 256 { 0 } else { h as u8 });
    ico.push(0); // colors
    ico.push(0); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // planes
    ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
    ico.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset
    ico.extend_from_slice(&png_bytes);

    fs::write(path, ico).expect("failed to write icon.ico");
}
