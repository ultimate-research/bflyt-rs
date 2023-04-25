use bflyt_rs::BflytFile;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bflyt = BflytFile::new_from_file("info_training.bflyt")?;
    
    // let output_path = "info_training.json";
    // let output = File::create(output_path)?;
    // serde_json::to_writer_pretty(output, &bflyt)?;

    // println!("Wrote out to {output_path}!");
    println!("{}", serde_json::to_string_pretty(&bflyt)?);
    Ok(())
}