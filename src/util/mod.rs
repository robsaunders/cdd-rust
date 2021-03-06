use std::path::PathBuf;

pub fn read_file(pathbuf: PathBuf) -> Result<String, failure::Error> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut f = File::open(pathbuf.clone()).map_err(|_| {
        failure::format_err!(
            "Could not open or read file: {}",
            pathbuf.to_str().unwrap_or("")
        )
    })?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;

    Ok(buffer)
}

pub fn write_file(pathbuf: PathBuf, content: &str) -> Result<(), failure::Error> {
    Ok(std::fs::write(pathbuf, content)?)
}

// pub fn file_exists<S: std::convert::AsRef<std::ffi::OsStr>>(filename: S) -> bool {
//     std::path::Path::new(&filename).exists()
// }

pub(crate) fn truncate(s: String, max_width: usize) -> String {
    s.chars().take(max_width).collect()
}
