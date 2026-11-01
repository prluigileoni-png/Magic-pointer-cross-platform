use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat};
use screenshots::Screen;
use std::io::Cursor;

pub fn capture_screen() -> Result<Vec<u8>> {
    let screens = Screen::all().context("enumerating displays")?;
    let screen = screens.first().context("no display available")?;
    let image = screen.capture().context("capturing primary display")?;
    let image = DynamicImage::ImageRgba8(image);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

pub fn run_platform_loop() -> Result<()> {
    #[cfg(target_os = "windows")]
    println!(
        "Windows capture adapter enabled. Global hotkeys/overlay require the desktop adapter."
    );
    #[cfg(target_os = "macos")]
    println!("macOS capture adapter enabled when Screen Recording permission is granted.");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    println!("This build provides provider/configuration functionality only.");
    Ok(())
}
