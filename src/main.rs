use bflyt_rs::BflytFile;
use std::fs::File;

use clap::Parser;

mod args;

use args::{Args, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.mode {
        Mode::Disasm { file, .. } => {
            let bflyt = BflytFile::new_from_file(file)?;
            let output_path = args.out.as_ref().map_or("out.yml", String::as_str);
            // let output = File::create(output_path)?;
            // serde_json::to_writer_pretty(output, &bflyt)?;
            // println!("Wrote out to {output_path}!");

            println!("{}", serde_json::to_string_pretty(&bflyt)?);
        }
        Mode::Asm { file, .. } => {
            let output_path = args.out.as_ref().map_or("out.bflyt", String::as_str);

            // let bflyt : BflytFile = serde_json::from_reader(File::open(file))?;

            // println!("{}", serde_json::to_string_pretty(&bflyt)?);
        }
    }

    Ok(())
}