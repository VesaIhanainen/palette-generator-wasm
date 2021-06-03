mod utils;

use std::str;

extern crate web_sys;
use web_sys::console;

use wasm_bindgen::prelude::*;
use crate::utils::set_panic_hook;


extern crate regex;
use regex::Regex;

use base64::decode;

extern crate kdtree;

use image::{GenericImageView,RgbImage, Rgb,Pixel};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use rand::Rng;

struct Color {
    rgb: [f64; 3],
}
struct Cluster {
    points: Vec<Color>,
}

const IMAGE_SIZE: usize = 64;
const OUTPUT_IMAGE_SIZE: usize = IMAGE_SIZE*IMAGE_SIZE*4;
static mut OUTPUT_IMAGE_BUFFER: [u8; OUTPUT_IMAGE_SIZE] = [0;OUTPUT_IMAGE_SIZE];

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn replace_data(input: &str) -> String{
    let mut output = String::with_capacity(input.len());
    let re: Regex = Regex::new("^data.*,").unwrap();
    output = re.replace(input, "").into();
    return output;
}

#[wasm_bindgen]
pub fn image(input: &[u8]){
    let _len = input.len();
}

#[wasm_bindgen]
pub fn image_output(input: &[u8]){
    set_panic_hook();
    let base64_string = match str::from_utf8(&input){
        Ok(v) => v,
        Err(e) => panic!("{}",e)
    };    
    let replace = replace_data(base64_string);
    let img_decoded: Vec<u8> = match decode(replace){
        Ok(v) => v,
        Err(e) => panic!("{}",e)
    };
    let img = match image::load_from_memory(img_decoded.as_slice()){
        Ok(v) => v,
        Err(e) => panic!("{}",e)
    };
    let k: u32 = 5;
    let centroids = return_colors(img, k, 10);
    let result_image = save_result(centroids,64);



    for x in 0..IMAGE_SIZE{
        for y in 0..IMAGE_SIZE{
            let linear_coordinate: usize = y * IMAGE_SIZE + x;
            let pixel = result_image.get_pixel(x as u32,y as u32).to_rgb();
            unsafe{
                OUTPUT_IMAGE_BUFFER[linear_coordinate*4 + 0] = pixel[0];
                OUTPUT_IMAGE_BUFFER[linear_coordinate*4 + 1] = pixel[1];
                OUTPUT_IMAGE_BUFFER[linear_coordinate*4 + 2] = pixel[2];
                OUTPUT_IMAGE_BUFFER[linear_coordinate*4 + 3] = 255;
            }
        }
    }
    
}

#[wasm_bindgen]
pub fn get_output_image_ptr() -> *const u8{
    let pointer: *const u8;
    unsafe{
        pointer = OUTPUT_IMAGE_BUFFER.as_ptr();
    }
    return pointer;
}

fn save_result(centroids: Vec<Color>, image_size: u32) -> RgbImage{
    let mut resulting_image = RgbImage::new(image_size,image_size);
    let mut index = 0;
    let num_centroids = centroids.len() as u32;
    for color in centroids {
        let r:u8 = (color.rgb[0] * 256.0_f64) as u8;
        let g:u8 = (color.rgb[1] * 256.0_f64) as u8;
        let b:u8 = (color.rgb[2] * 256.0_f64) as u8;
        for x in index*(image_size-1)/num_centroids..=(index+1)*(image_size-1)/num_centroids {
            for y in 0..=(image_size-1){
                resulting_image.put_pixel(x,y,Rgb([r,g,b]));
            }
        }
        index = index + 1;
    }
    return resulting_image;
}
fn return_colors(input_image: image::DynamicImage, k: u32, runs: i32) -> Vec<Color> {
    let mut centroids = random_centroids(k);
    let image_width = input_image.dimensions().0 as usize;
    let image_height = input_image.dimensions().1 as usize;
    let image_vec = input_image.as_rgb8().unwrap().to_vec();
    let mut clusters = init_clusters(k);
    let mut ran = 0;
    while ran < runs {
        let mut kdtree = KdTree::<f64, usize, &[f64; 3]>::new(3);
        let mut clusters_pos = vec![0; k as usize];
        for i in 0..(k as usize) {
            kdtree.add(&centroids[i].rgb, i).unwrap();
        }
        let mut closest = Vec::<usize>::new();
        for x in 0..image_width {
            for y in 0..image_height {
                let index = (x * image_height * 3 + y * 3) as usize;
                let r = image_vec[index + 0] as f64 / 256.0_f64;
                let g = image_vec[index + 1] as f64 / 256.0_f64;
                let b = image_vec[index + 2] as f64 / 256.0_f64;
                let a = kdtree.nearest(&[r, g, b], 1, &squared_euclidean).unwrap();
                closest.push(*a[0].1);
            }
        }
        let mut histogram = vec![0; k as usize];
        for x in closest.iter() {
            histogram[*x as usize] += 1;
        }
        for x in 0..image_width {
            for y in 0..image_height {
                let index2d = (y + image_height * x) as usize;
                let cluster_n = closest[index2d] as usize;

                let index = (x * image_height * 3 + y * 3) as usize;
                let r = image_vec[index] as f64 / 256.0_f64;
                let g = image_vec[index + 1] as f64 / 256.0_f64;
                let b = image_vec[index + 2] as f64 / 256.0_f64;
                clusters[cluster_n].points.push(Color { rgb: [r, g, b] });
                clusters_pos[cluster_n] += 1;
            }
        }
        for i in 0..(k as usize) {
            let mut rgb = Color {
                rgb: [0.0_f64, 0.0_f64, 0.0_f64],
            };
            let mut times: i32 = 0;
            let points_len = clusters[i].points.len();
            for ind in 0..points_len {
                let clus_rgb = Color {
                    rgb: [
                        clusters[i].points[ind].rgb[0],
                        clusters[i].points[ind].rgb[1],
                        clusters[i].points[ind].rgb[2],
                    ],
                };
                if 1E-4 < clus_rgb.rgb[0] && 1E-4 < clus_rgb.rgb[1] && 1E-4 < clus_rgb.rgb[2] {
                    times += 1;
                    rgb.rgb[0] += clus_rgb.rgb[0];
                    rgb.rgb[1] += clus_rgb.rgb[1];
                    rgb.rgb[2] += clus_rgb.rgb[2];
                }
            }
            if times == 0 {
                centroids[i].rgb[0] = rgb.rgb[0];
                centroids[i].rgb[1] = rgb.rgb[1];
                centroids[i].rgb[2] = rgb.rgb[2];
            } else {
                centroids[i].rgb[0] = rgb.rgb[0] / (times as f64);
                centroids[i].rgb[1] = rgb.rgb[1] / (times as f64);
                centroids[i].rgb[2] = rgb.rgb[2] / (times as f64);
            }
            println!(
                "{:?}",
                (
                    centroids[i].rgb[0],
                    centroids[i].rgb[1],
                    centroids[i].rgb[2]
                )
            );
        }
        for x in 0..k as usize {
            clusters[x].points.clear();
        }
        ran += 1;
    }
    return centroids;
}

fn random_centroids(k: u32) -> Vec<Color> {
    let mut centroids = Vec::<Color>::new();
    let mut rng = rand::thread_rng();
    for _i in 0..k {
        let r = (rng.gen_range(1..255) as f64) / 256.0_f64;
        let b = (rng.gen_range(1..255) as f64) / 256.0_f64;
        let g = (rng.gen_range(1..255) as f64) / 256.0_f64;

        let a = Color { rgb: [r, g, b] };
        centroids.push(a);
    }
    return centroids;
}

fn init_clusters(k: u32) -> Vec<Cluster> {
    let mut clusters = Vec::<Cluster>::new();
    for _i in 0..k {
        let new_cluster = Cluster {
            points: Vec::<Color>::new(),
        };
        clusters.push(new_cluster);
    }
    return clusters;
}
