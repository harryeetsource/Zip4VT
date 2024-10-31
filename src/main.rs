use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use log::{info, warn};
use simplelog::*;
use std::fs;
use std::io::{Seek, SeekFrom};

const MAX_ZIP_SIZE: u64 = 450 * 1024 * 1024; // 450MB

fn main() -> std::io::Result<()> {
    // Initialize the logger
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("zip_progress.log")?),
        ]
    ).unwrap();

    let dir_path = "C:\\Users\\WDAGUtilityAccount\\Desktop\\malware\\0B9ED125FC0241E7997230BE025F8CA0"; // Change this to your directory
    let mut zip_index = 1;
    let mut buffer = Vec::new();

    let zip_file_name = |index: usize| format!("archive_part_{}.zip", index);

    let mut create_new_zip = |index: usize| -> zip::result::ZipResult<(File, zip::ZipWriter<File>)> {
        let zip_file = File::create(zip_file_name(index))?;
        let zip = zip::ZipWriter::new(zip_file.try_clone()?);
        info!("Started new ZIP file: {}", zip_file_name(index));
        Ok((zip_file, zip))
    };

    let (mut zip_file, mut zip) = create_new_zip(zip_index)?;
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated); // You can change the compression method

    info!("Starting to walk through directory: {}", dir_path);

    for entry in WalkDir::new(dir_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read entry: {}", e);
                continue;
            }
        };
        let path = entry.path();

        if path.is_file() {
            let file_size = entry.metadata()?.len();
            info!("Processing file: {} (size: {} bytes)", path.display(), file_size);

            // Add the file to the ZIP
            let name_in_zip = path.strip_prefix(Path::new(dir_path)).unwrap();
            zip.start_file(name_in_zip.to_string_lossy().replace("\\", "/"), options)?;

            let mut f = File::open(path)?;
            buffer.clear();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;

            // Ensure the ZIP file size is limited to 450MB
            zip.flush()?; // Ensure all data is written to the zip_file

            zip_file.sync_all()?; // Sync all data to disk to get the correct file size
            let current_zip_size = zip_file.metadata()?.len();
            info!("Current ZIP size: {} bytes", current_zip_size);

            if current_zip_size > MAX_ZIP_SIZE {
                info!("Current ZIP file exceeded {} MB, finishing and creating a new ZIP file", MAX_ZIP_SIZE / (1024 * 1024));
                zip.finish()?;
                zip_index += 1;
                let (new_zip_file, new_zip) = create_new_zip(zip_index)?;
                zip_file = new_zip_file;
                zip = new_zip;
            }
        }
    }

    // Finish the last ZIP file
    zip.finish()?;
    info!("Finished creating ZIP files.");

    Ok(())
}

// Helper function to calculate the size of a file
fn get_file_size(path: &Path) -> u64 {
    match path.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    }
}
