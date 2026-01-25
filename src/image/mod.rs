use image::{GenericImageView, Pixel};
use std::path::Path;


/// Load an image file into a 2D bitmap buffer.
///
/// The image is loaded using the `image` crate and converted into
/// a `Vec<Vec<u32>>` where each pixel is stored in ARGB format.
///
/// # Panics
/// Panics if the image cannot be opened or decoded.
pub fn load_image_2d<P: AsRef<Path>>(path: P) -> image::ImageResult<Vec<Vec<u32>>> {
    let img = image::open(path)?;
    let (width, height) = img.dimensions();

    let mut buffer = vec![vec![0u32; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y).to_rgb();
            let [r, g, b] = pixel.0;

            buffer[y as usize][x as usize] =
                ((r as u32) << 16) |
                ((g as u32) << 8)  |
                (b as u32);
        }
    }

    Ok(buffer)
}

/// Loads a sprite sheet image and slices it into individual sprites.
///
/// The image is divided into a grid of equally sized sprites, where each sprite
/// has a fixed width and height in pixels. Sprites are extracted left-to-right,
/// top-to-bottom, and returned as a flat vector.
///
/// Each sprite is represented as a 2D pixel buffer (`Vec<Vec<u32>>`) in
/// row-major order. Pixel values are encoded as `0xRRGGBB`.
///
/// # Parameters
/// - `path`: Path to the sprite sheet image file.
/// - `sprite_width`: Width of a single sprite in pixels.
/// - `sprite_height`: Height of a single sprite in pixels.
///
/// # Returns
/// - `Ok(Vec<Vec<Vec<u32>>>)` containing all extracted sprites.
///   - Outer `Vec`: list of sprites
///   - Middle `Vec`: rows of a sprite (Y axis)
///   - Inner `Vec`: pixels in a row (X axis)
///
/// # Errors
/// Returns an `image::ImageError` if the image cannot be loaded or decoded.
///
/// # Notes
/// - If the image dimensions are not evenly divisible by `sprite_width` or
///   `sprite_height`, any leftover pixels on the right or bottom edges are ignored.
/// - Alpha channels are discarded; only RGB data is used.
///
/// # Example
/// ```no_run
/// let sprites = load_sprite_sheet("sprites.png", 32, 64)?;
/// let first_sprite = &sprites[0];
/// ```
pub fn load_sprite_sheet<P: AsRef<Path>>(
    path: P,
    sprite_width: u32,
    sprite_height: u32,
) -> image::ImageResult<Vec<Vec<Vec<u32>>>> {
    let img = image::open(path)?;
    let (sheet_width, sheet_height) = img.dimensions();

    let mut sprites = Vec::new();

    let sprites_x = sheet_width / sprite_width;
    let sprites_y = sheet_height / sprite_height;

    for sy in 0..sprites_y {
        for sx in 0..sprites_x {
            let mut sprite = vec![vec![0u32; sprite_width as usize]; sprite_height as usize];

            for y in 0..sprite_height {
                for x in 0..sprite_width {
                    let px = sx * sprite_width + x;
                    let py = sy * sprite_height + y;

                    let pixel = img.get_pixel(px, py).to_rgb();
                    let [r, g, b] = pixel.0;

                    sprite[y as usize][x as usize] =
                        ((r as u32) << 16) |
                        ((g as u32) << 8)  |
                        (b as u32);
                }
            }

            sprites.push(sprite);
        }
    }

    Ok(sprites)
}

