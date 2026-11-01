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
