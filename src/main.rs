use bflyt_rs::BflytFile;

fn main() -> Result<(), std::io::Error> {
    BflytFile::new_from_file("info_training.bflyt")
}