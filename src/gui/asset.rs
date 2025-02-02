//! Asset (resource) manager for GUI.
//!
//! This module provides facilities to access texture data.

use include_dir::{include_dir, Dir};

/// The embedded filesystem.
static GUI_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/asset/gui");

/// Parameters for a [`load_asset`] call.
///
/// The resource requested is `{location}/{name}{suffix}`.
struct AssetLoadRequest<'a> {
    /// A human-readable description of what is being loaded in Sentence case, e.g. `Image`.
    kind: &'a str,

    /// The directory to load the asset from without the trailing slash.
    location: &'a str,

    /// The name of the asset to load.
    name: &'a str,

    /// A suffix appened to [`name`](Self::name) to form a complete path, e.g. `.png`.
    suffix: &'a str,
}

/// Provides a [`std::io::Cursor`] view of an asset.
///
/// The [`name`](AssetLoadRequest::name) parameter must match regex `[A-Za-z_]+`, otherwise this
/// function panics. No other parameters are checked.
///
/// If the asset could not be found, this function panics.
fn load_asset(req: AssetLoadRequest) -> std::io::Cursor<&[u8]> {
    let AssetLoadRequest {
        kind,
        location,
        name,
        suffix,
    } = req;

    assert!(
        name.chars()
            .all(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '_'))
            && !name.is_empty(),
        "{kind} name is not allowed: {name:?}"
    );

    let file = GUI_ASSETS
        .get_file(format!("{location}/{name}{suffix}"))
        .expect(&format!("{kind} {name:?} not found"));

    std::io::Cursor::new(file.contents())
}

/// Loads an image by its name.
///
/// No caching takes place - each successful call results in a new allocation and decoding.
///
/// The name must match regex `[A-Za-z_]+`, otherwise this function panics.
///
/// Missing data, IO errors, decoding errors, allocation errors all result in a panic.
pub fn load_image(name: &str) -> image::DynamicImage {
    let cursor = load_asset(AssetLoadRequest {
        kind: "Image",
        location: "image",
        name,
        suffix: ".png",
    });

    image::load(cursor, image::ImageFormat::Png)
        .expect(&format!("Image {name:?} is not a valid PNG file"))
}

/// Loads an 3D mesh by its name, returning it as a [`super::Mesh`].
///
/// No caching takes place - each successful call results in a new allocation and decoding.
///
/// The name must match regex `[A-Za-z_]+`, otherwise this function panics.
///
/// Missing data, IO errors, decoding errors, allocation errors all result in a panic.
pub fn load_mesh(name: &str) -> super::Mesh {
    let cursor = load_asset(AssetLoadRequest {
        kind: "Mesh",
        location: "mesh",
        name,
        suffix: ".obj",
    });

    super::Mesh::load_obj(cursor)
        .expect(&format!("Mesh {name:?} is not a valid OBJ file"))
}
