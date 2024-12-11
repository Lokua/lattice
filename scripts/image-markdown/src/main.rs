use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Must be run from the lattice project root

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let images_dir = Path::new("images");
    let output_file = Path::new("index.md");

    // Get all image files sorted by modification time
    let mut image_paths: Vec<PathBuf> = WalkDir::new(images_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_supported_image(e.path()))
        .map(|e| e.path().to_owned())
        .collect();

    image_paths.sort_by(|a, b| {
        fs::metadata(b)
            .unwrap()
            .modified()
            .unwrap()
            .cmp(&fs::metadata(a).unwrap().modified().unwrap())
    });

    // Generate markdown content
    let mut markdown_content =
        String::from("Files sorted from most to least recent\n\n");

    for path in image_paths {
        let filename = path.file_name().unwrap().to_string_lossy();

        // Read and resize image
        let img = image::open(&path)?;
        let resized =
            img.resize(1000, 1000, image::imageops::FilterType::Lanczos3);

        // Convert to PNG in memory
        let mut base64_data = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut base64_data), ImageFormat::Png)?;
        let base64_string = STANDARD.encode(&base64_data);

        // Add to markdown
        markdown_content.push_str(&format!("## {}\n", filename));
        markdown_content.push_str(&format!(
            "<img alt=\"{}\" width=\"500\" src=\"data:image/png;base64,{}\">\n\n",
            filename, base64_string
        ));
    }

    // Write to file
    let mut file = File::create(output_file)?;
    file.write_all(markdown_content.as_bytes())?;

    Ok(())
}

fn is_supported_image(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(
            extension.to_string_lossy().to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif"
        )
    } else {
        false
    }
}
