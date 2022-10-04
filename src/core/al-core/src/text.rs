#[derive(Serialize, Deserialize)]
pub struct LetterTexPosition {
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
    pub x_advance: u32,
    pub y_advance: u32,
    pub w: u32,
    pub h: u32,
    pub bound_xmin: f32,
    pub bound_ymin: f32,
}

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
pub struct Font {
    pub bitmap: Vec<u8>,
    pub letters: HashMap<char, LetterTexPosition>,
}

pub const TEX_SIZE: usize = 256;

mod tests {
    #[test]
    pub fn rasterize_font() {
        #[derive(PartialEq)]
        struct Letter {
            pub l: char,
            pub w: u32,
            pub h: u32,
            pub x_advance: u32,
            pub y_advance: u32,
            pub bitmap: Vec<u8>,
            pub bounds: fontdue::OutlineBounds,
        }
        use std::cmp::Ordering;
        impl PartialOrd for Letter {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                let w_cmp = other.w.cmp(&self.w);
        
                if Ordering::Equal == w_cmp {
                    Some(other.h.cmp(&self.h))
                } else {
                    Some(w_cmp)
                }
            }
        }
        
        use super::TEX_SIZE;

        use super::LetterTexPosition;
        use std::collections::HashMap;
        use std::io::Write;


        // Read the font data.
        let font = include_bytes!("../resources/arial.ttf") as &[u8];
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        // Rasterize and get the layout metrics for the letter 'g' at 17px.
        let mut w = 0;
        let mut h = 0;
        let mut letters = Vec::new();
        for c in 0_u8..255_u8 {
            let (metrics, bitmap) = font.rasterize(c as char, 16.0);

            letters.push(Letter {
                w: metrics.width as u32,
                h: metrics.height as u32,
                x_advance: metrics.advance_width as u32,
                y_advance: metrics.advance_height as u32,
                bounds: metrics.bounds,
                l: c as char,
                bitmap,
            });

            h += metrics.height;
            w = std::cmp::max(w, metrics.width);
        }
        letters.sort_unstable_by(|l, r| {
            let w_cmp = r.w.cmp(&l.w);

            if Ordering::Equal == w_cmp {
                r.h.cmp(&l.h)
            } else {
                w_cmp
            }
        });

        let mut letters_tex = HashMap::new();
        let mut x_min = 0;
        let mut y_min = 0;
        let mut size_col = letters[0].w;
        let mut img = vec![0; TEX_SIZE * TEX_SIZE * 4];

        for Letter {
            l,
            w,
            h,
            x_advance,
            y_advance,
            bitmap,
            bounds,
        } in letters.into_iter()
        {
            let mut i = 0;

            let mut y_max = y_min + h;
            if y_max >= TEX_SIZE as u32 {
                y_min = 0;
                y_max = h;
                x_min += size_col;

                size_col = w;
            }

            // Draw here the letter in the tex
            let x_max = x_min + w;
            letters_tex.insert(
                l,
                LetterTexPosition {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    x_advance,
                    y_advance,
                    w: x_max - x_min,
                    h: y_max - y_min,
                    bound_xmin: bounds.xmin,
                    bound_ymin: bounds.ymin,
                },
            );
            for y in (y_min as usize)..(y_max as usize) {
                for x in (x_min as usize)..(x_max as usize) {
                    img[4 * (x + TEX_SIZE * y)] = bitmap[i];
                    img[4 * (x + TEX_SIZE * y) + 1] = bitmap[i];
                    img[4 * (x + TEX_SIZE * y) + 2] = bitmap[i];
                    img[4 * (x + TEX_SIZE * y) + 3] = bitmap[i];

                    i += 1;
                }
            }

            y_min += h;
        }

        /* Save the jpeg file */
        use std::fs::File;
        use std::io::BufWriter;

        let file = File::create("letters.png").unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, TEX_SIZE as u32, TEX_SIZE as u32); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&img).unwrap(); // Save

        /* Save the letters position */
        let letters_tex_serialized = serde_json::to_string(&letters_tex).unwrap();

        let mut file = File::create("letters.json").unwrap();
        write!(file, "{}", letters_tex_serialized).unwrap();
    }
}
