use std::{error::Error, fs};
use walkdir::WalkDir;
extern crate walkdir;
use std::io::BufRead;

// All my shaders reside in the 'src/shaders' directory
fn generate_shaders() -> std::result::Result<(), Box<dyn Error>> {
    println!("generate shaders");
    let mut shaders = HashMap::new();

    for entry in WalkDir::new("../glsl/webgl2/")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "vert" || ext == "frag" {
                    let file_name = path.file_name().unwrap().to_str().unwrap();

                    let out_file_name = path
                        .strip_prefix("../glsl/webgl2/")
                        .unwrap()
                        //.with_extension("")
                        .to_string_lossy()
                        .to_owned()
                        .replace("/", "_");
                    //let out_name = format!("{}/{}", OUT_PATH, out_file_name);

                    let src = read_shader(path)?;
                    shaders.insert(out_file_name, src);

                    //fs::write(&out_name, result)?;
                    println!("cargo:rerun-if-changed=src/shaders/{}", file_name);
                }
            }
        }
    }

    write("src/shaders.rs".into(), shaders)?;

    Ok(())
}

fn read_shader<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    let path = path.as_ref();
    let file = fs::File::open(path.to_str().unwrap())?;

    let shader_src = std::io::BufReader::new(file)
        .lines()
        .flatten()
        .map(|l| {
            if l.starts_with("#include") {
                let incl_file_names: Vec<_> = l.split_terminator(&[';', ' '][..]).collect();
                let incl_file_name_rel = incl_file_names[1];
                let incl_file_name = path.parent().unwrap().join(incl_file_name_rel);

                read_shader(incl_file_name.to_str().unwrap()).unwrap()
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(shader_src)
}

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn write(path: PathBuf, entries: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let mut all_the_files = File::create(&path)?;

    writeln!(&mut all_the_files, r#"use std::collections::HashMap;"#,)?;
    writeln!(&mut all_the_files, r#""#,)?;
    writeln!(&mut all_the_files, r#"#[allow(dead_code)]"#,)?;
    writeln!(
        &mut all_the_files,
        r#"pub fn get_all() -> HashMap<&'static str, &'static str> {{"#,
    )?;
    writeln!(&mut all_the_files, r#"    let mut out = HashMap::new();"#,)?;

    for (name, content) in entries {
        writeln!(
            &mut all_the_files,
            r##"    out.insert("{name}", r#"{content}"#);"##,
        )?;
    }

    writeln!(&mut all_the_files, r#"    out"#,)?;
    writeln!(&mut all_the_files, r#"}}"#,)?;

    Ok(())
}

fn main() {
    if let Err(err) = generate_shaders() {
        // panic here for a nicer error message, otherwise it will
        // be flattened to one line for some reason
        panic!("Unable to generate shaders\n{}", err);
    }
}
