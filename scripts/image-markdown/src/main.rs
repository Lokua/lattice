use image::ImageFormat;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Must be run from the project root

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let images_dir = Path::new("images");
    let resized_dir = images_dir.join("1000x");
    let output_file = Path::new("index.md");

    // Create resized images directory if it doesn't exist
    fs::create_dir_all(&resized_dir)?;

    // Get all image files sorted by modification time
    let mut image_paths: Vec<PathBuf> = WalkDir::new(images_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_supported_image(e.path()))
        // Exclude images in the 1000x directory
        .filter(|e| !e.path().starts_with(&resized_dir))
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
        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
        let resized_path = resized_dir.join(&filename);

        // Only resize if the resized version doesn't exist or is older
        if !resized_path.exists() || is_source_newer(&path, &resized_path)? {
            println!("Resizing {}", filename);

            // Read and resize image
            let img = image::open(&path)?;
            let resized =
                img.resize(1000, 1000, image::imageops::FilterType::Lanczos3);

            // Save resized image
            resized.save_with_format(&resized_path, ImageFormat::Png)?;
        }

        // Add to markdown
        markdown_content.push_str(&format!("## {}\n\n", filename));
        markdown_content.push_str(&format!(
            "<img src=\"images/1000x/{}\" alt=\"{}\" width=\"500\">\n\n",
            filename, filename
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

fn is_source_newer(
    source: &Path,
    resized: &Path,
) -> Result<bool, std::io::Error> {
    let source_modified = fs::metadata(source)?.modified()?;
    let resized_modified = fs::metadata(resized)?.modified()?;
    Ok(source_modified > resized_modified)
}
