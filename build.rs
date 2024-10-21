use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("rooster_sound.rs");
    let mut f = File::create(&dest_path).unwrap();

    let rooster_mp3 = include_bytes!("rooster.mp3");
    
    writeln!(&mut f, "pub const ROOSTER_SOUND: &[u8] = &{:?};", rooster_mp3).unwrap();
}
