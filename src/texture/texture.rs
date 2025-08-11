use image::DynamicImage;
use wgpu::*;

pub struct RgbaTexture {
    #[allow(unused)]
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl RgbaTexture {
    pub fn from_image(device: &Device, queue: &Queue, image: DynamicImage) -> RgbaTexture {
        let image = image.into_rgba8();

        let size = image.dimensions();
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("RgbaTexture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            image.as_ref(),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor::default());

        RgbaTexture {
            texture,
            view,
            sampler,
        }
    }
}
