use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};

/// Lighter color used by [`quantize_binary`].
pub const COLOR_LIGHT: Rgba<u8> = Rgba([255, 255, 255, 0]);

/// Darker color used by [`quantize_binary`].
pub const COLOR_DARK: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Applies a simplistic binary (light or dark) color
/// quantization (aka "reduction" or "posterization")
/// to `image`, returning a new image where each pixel
/// is either [`COLOR_LIGHT`] or [`COLOR_DARK`]
pub fn quantize_binary(image: &DynamicImage) -> DynamicImage {
    let mut new_image = image.clone();

    for (x, y, color) in image.pixels() {
        if color == COLOR_LIGHT || color == COLOR_DARK {
            new_image.put_pixel(x, y, color);
            continue;
        }

        let sum: u32 = color.0[0] as u32 + color.0[1] as u32 + color.0[2] as u32;
        if sum >= (u8::MAX as f32 * 1.75f32) as u32 {
            new_image.put_pixel(x, y, COLOR_LIGHT);
        } else {
            new_image.put_pixel(x, y, COLOR_DARK);
        }
    }

    new_image
}

/// Applies a naive dimetric ("2:1 isometric") distortion
/// to an image, returning a new image with its perspective
/// projected into a dimetric plane.
///
/// Related reading:
///
/// - https://screamingbrainstudios.com/making-isometric-tiles/
/// - https://screamingbrainstudios.com/cartesian-transform/
/// - https://stackoverflow.com/questions/2446494/skewing-an-image-using-perspective-transforms
/// - https://www.mbeckler.org/inkscape/isometric_projection/
pub fn distort_dimetric(image: &DynamicImage) -> DynamicImage {
    let mut new_image =
        DynamicImage::ImageRgba8(ImageBuffer::new(image.width() * 2, image.height()));

    for (x, y, color) in image.pixels() {
        let x = x as i64;
        let y = y as i64;

        let new_x = (x - y) + (new_image.width() as i64 / 2);
        let new_y = (x + y) / 2;

        if new_x >= 0
            && new_y >= 0
            && (new_x as u32) < new_image.width()
            && (new_y as u32) < new_image.height()
        {
            new_image.put_pixel(new_x as u32, new_y as u32, color);
        } else {
            new_image.put_pixel(
                new_x as u32 % new_image.width(),
                new_y as u32 % new_image.height(),
                color,
            );
        }
    }

    new_image
}
