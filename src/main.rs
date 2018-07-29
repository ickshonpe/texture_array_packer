extern crate image;
extern crate rect_packer;
extern crate serde;
extern crate serde_json;

use rect_packer::{Packer, Rect};
use image::{DynamicImage, GenericImage};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;



fn load_images(path_name: &Path) -> Vec<(String, DynamicImage)> {
    let dir = match std::fs::read_dir(path_name) {
        Ok(dir) => { dir },
        Err(e) => {
            eprintln!("Path not found '{:?}'.\n\n {:?}", path_name, e);
            std::process::exit(2);
        }
    };
    let mut source_images: Vec<(String, DynamicImage)> = Vec::new();
    for dir_entry in dir {
        if let Ok(entry) = dir_entry {
            if entry.file_type().unwrap().is_file() {
                let path = entry.path();
                if  path.extension() == Some(OsStr::new("png")) {
                    let input_buffer = image::open(&path).unwrap();
                    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    source_images.push((name, input_buffer));
                }
            }
        }
    }
    source_images
}

fn pack_rects(layer_size: (u32, u32), images: Vec<(String, DynamicImage)>) -> (Vec<(String, DynamicImage, Option<Rect>, u32)>, u32) {
    let mut output_buffer = Vec::<(String, DynamicImage, Option<Rect>, u32)>::with_capacity(images.len());
    let config = rect_packer::Config {
        width: layer_size.0 as i32,
        height: layer_size.1 as i32,
        border_padding: 0,
        rectangle_padding: 0
    };
    let mut layer_count = 0;
    let mut layers = Vec::<Packer>::new();
    'outer: for (name, image) in images {
        for (index, layer) in layers.iter_mut().enumerate() {
            if let Some(r) = layer.pack(image.width() as i32, image.height() as i32, false) {
                let out = (name, image, Some(r), index as u32);
                output_buffer.push(out);
                continue 'outer;
            }
        }
        let mut new_layer = Packer::new(config);
        let r  = new_layer.pack(image.width() as i32, image.height() as i32, false);
        let out = (name, image, r, layers.len() as u32);
        layers.push(new_layer);
        output_buffer.push(out);
        layer_count += 1;
    }
    (output_buffer, layer_count)
}

type OutputManifest = std::collections::HashMap<String, (u32, u32, u32, u32, u32)>;

fn create_output_image(image_packing: Vec<(String, DynamicImage, Rect, u32)>, layer_size: (u32, u32), total_layers: u32) -> (image::RgbaImage, OutputManifest) {
    let output_size = (layer_size.0, layer_size.1 * total_layers);
    let mut output_manifest = OutputManifest::new();
    let mut output_buffer: image::RgbaImage = image::ImageBuffer::new(output_size.0, output_size.1);
    for (name, image, rect, layer) in image_packing {
        println!("writing {}\n\t {:?}\n\t layer: {}", name, rect, layer);
        let target_x = rect.x as u32;
        let target_y = rect.y as u32 + (layer * layer_size.1);
        for x in 0..image.width() {
            for y in 0..image.height() {
                let in_pixel = image.get_pixel(x, y);
                output_buffer.put_pixel(target_x + x, target_y + y, in_pixel);
            }
        }
        output_manifest.insert(name, (rect.x as u32, rect.y as u32, rect.width as u32, rect.height as u32, layer));
    }
    (output_buffer, output_manifest)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let path_name = if args.len() > 1 { &args[1] } else { "." };
    let images = load_images(Path::new(path_name));
    let size =
        if args.len() > 2 {
            args[2].parse::<u32>().unwrap()
        } else {
            1024
        };


    let layer_size = (size, size);

    let (packing, layer_count) = {
        let (mut packing, layer_count) = pack_rects(layer_size, images);
        for (name, _,r, _) in &packing {
            if r.is_none() {
                println!("\t{} was not packed", name);
            }
        }
        (packing.into_iter().filter_map(|p| {
            if let Some(r) = p.2 {
                Some((p.0, p.1, r, p.3))
            } else {
                None
            }
        }).collect(), layer_count)
    };
    let (output_image, output_manifest) = create_output_image(packing, layer_size, layer_count);
    let _ = image::ImageRgba8(output_image).save(&Path::new("texture_array.png"));
    let serialized_manifest = serde_json::to_string(&output_manifest).unwrap();
    let ref mut manifest_output_file = File::create(&Path::new("texture_array.json")).unwrap();
    let _ = manifest_output_file.write_all(serialized_manifest.as_bytes());


}
