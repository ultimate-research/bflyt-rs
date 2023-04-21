use bflyt_rs::BflytFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BflytFile::new_from_file("info_training.bflyt")
}