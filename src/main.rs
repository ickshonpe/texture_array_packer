extern crate rect_packer;
extern crate serde;
extern crate image;

use rect_packer::Packer;
use image::DynamicImage;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;

pub struct SrcRect {
    pub name: String,
    pub width: i32,
    pub height: i32
}

fn load_images() -> Vec<(String, DynamicImage)> {
    let args = std::env::args().collect::<Vec<_>>();
    let path_name = if args.len() > 1 { &args[1] } else { "." };
    let dir = match std::fs::read_dir(path_name) {
        Ok(dir) => { dir },
        Err(e) => {
            println!("Path not found '{}'.\n\n {:?}", path_name, e);
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


fn main() {
    let images = vec![
        SrcRect {
            name: "block".into(),
            width: 32,
            height: 32,
        },
        SrcRect {
            name: "charset".into(),
            width: 512,
            height: 512,
        },
        SrcRect {
            name: "block2".into(),
            width: 320,
            height: 320,
        },
        SrcRect {
            name: "block3".into(),
            width: 3200,
            height: 320,
        },
        SrcRect {
            name: "sagan".into(),
            width: 600,
            height: 600,
        },
        SrcRect {
            name: "milk".into(),
            width: 1000,
            height: 1000,
        },
        SrcRect {
            name: "poop".into(),
            width: 500,
            height: 600,
        },
        SrcRect {
            name: "poop2".into(),
            width: 600,
            height: 400,
        },
    ];
    let layer_size = (1024, 1024);
    let config = rect_packer::Config {
        width: layer_size.0,
        height: layer_size.1,
        border_padding: 0,
        rectangle_padding: 0
    };

    let mut output_buffer = Vec::with_capacity(images.len());
    let mut layers = Vec::<Packer>::new();

    for src_rect in &images {
        println!("placing {}", src_rect.name);
        let mut success = false;
        for (index, layer) in layers.iter_mut().enumerate() {
            if let Some(r) = layer.pack(src_rect.width, src_rect.height, false) {
                println!("\t Image {} pushed to layer {}", src_rect.name, index);
                output_buffer.push((src_rect.name.clone(), r, index));
                success = true;
                break;
            }
        }
        if !success {
            println!("\tplacing {} in new layer", src_rect.name);
            let mut new_layer = Packer::new(config);
            if let Some(r) = new_layer.pack(src_rect.width, src_rect.height, false) {
                layers.push(new_layer);
                println!("\t Image {} pushed to layer {}", src_rect.name, layers.len() - 1);
                output_buffer.push((src_rect.name.clone(), r, layers.len() - 1));
            } else {
                println!("\tSource image {} too large to fit in new layer, ignored.", src_rect.name);
            }

        }
    }


}
