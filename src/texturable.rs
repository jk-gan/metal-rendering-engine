use image;
use image::error::{ImageError, ImageResult, UnsupportedError};
use image::GenericImageView;
use metal::*;
use png;
use png::OutputInfo;
use std::fs::File;

pub trait Texturable {
    fn load_texture(image_name: &str, device: &Device) -> ImageResult<Texture> {
        let image_name = match image_name.contains(".png") {
            true => image_name.to_string(),
            false => format!("{}.png", image_name),
        };
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("resources/{}", image_name));

        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        match img {
            image::DynamicImage::ImageRgb8(img) => {
                let texture_descriptor = TextureDescriptor::new();
                texture_descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
                texture_descriptor.set_width(width as u64);
                texture_descriptor.set_height(height as u64);
                // texture_descriptor.set_depth(1);
                texture_descriptor.set_mipmap_level_count_for_size(MTLSize {
                    width: width as u64,
                    height: height as u64,
                    depth: 1,
                });

                let texture = device.new_texture(&texture_descriptor);

                let mut new_buf: Vec<u8> = vec![];

                for row in img.rows().rev() {
                    for pixel in row {
                        new_buf.push(pixel[2]);
                        new_buf.push(pixel[1]);
                        new_buf.push(pixel[0]);
                        new_buf.push(255);
                    }
                }

                let region = MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width: width as u64,
                        height: height as u64,
                        depth: 1,
                    },
                };
                texture.replace_region(region, 0, new_buf.as_ptr() as _, width as u64 * 4);
                Ok(texture)
            }
            _ => todo!(),
        }
    }
}
