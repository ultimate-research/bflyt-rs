use bflyt_rs::BflytFile;
use std::fs::File;

use clap::Parser;

mod args;

use args::{Args, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.mode {
        Mode::Unpack { file, .. } => {
            let bflyt = BflytFile::new_from_file(file)?;
            let output_path = args.out.as_ref().map_or("out.json", String::as_str);
            let output = File::create(output_path)?;
            if args.print.is_some() && args.print.unwrap() {
                println!("{}", serde_json::to_string_pretty(&bflyt)?);
            } else {
                serde_json::to_writer_pretty(output, &bflyt)?;
                println!("Wrote out to {output_path}!");
            }
        }
        Mode::Pack { file, .. } => {
            let output_path = args.out.as_ref().map_or("out.bflyt", String::as_str);
            let input_json = std::fs::read_to_string(file)?;
            let bflyt : BflytFile = serde_json::from_str(&input_json)?;

            bflyt.write_to_file(output_path)?;
        }
    }

    Ok(())
}
