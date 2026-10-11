use std::path::Path;

use image::GenericImageView;

pub fn compare_image_snapshot(
    actual_img: &image::RgbaImage,
    snapshot_dir: &Path,
    output_dir: &Path,
    name: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(output_dir).unwrap();
    let actual_file = output_dir.join(format!("{name}.png"));
    actual_img.save(&actual_file).unwrap();

    let expected_file = snapshot_dir.join(format!("{name}.png"));
    let expected_img = image::open(&expected_file).map_err(|error| {
        format!(
            "Failed to read snapshot {}: {error}. Actual image: {}",
            expected_file.display(),
            actual_file.display()
        )
    })?;

    if actual_img.dimensions() != expected_img.dimensions() {
        return Err(format!(
            "Image dimensions are different. snapshot: {}, expected: {:?}, actual: {:?}",
            expected_file.display(),
            expected_img.dimensions(),
            actual_img.dimensions()
        ));
    }

    let (image_width, image_height) = actual_img.dimensions();
    let mut img_buf = image::RgbaImage::new(image_width, image_height);
    let mut diff = false;

    for y in 0..image_height {
        for x in 0..image_width {
            let actual_pixel = *actual_img.get_pixel(x, y);
            let expected_pixel = expected_img.get_pixel(x, y);

            if actual_pixel != expected_pixel {
                img_buf.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                diff = true;
            } else {
                img_buf.put_pixel(x, y, actual_pixel);
            }
        }
    }

    if diff {
        let diff_file = output_dir.join(format!("{name}_diff.png"));
        img_buf.save(&diff_file).unwrap();
        return Err(format!(
            "Images are different. diff: {}",
            diff_file.display()
        ));
    }

    Ok(())
}
