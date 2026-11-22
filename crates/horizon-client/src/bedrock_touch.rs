// Adapted from Starling 5432370d573642041585c69b40fb045b27133488 (MIT).
// Copyright (c) 2026 Ludovic Marie and the Horizon contributors.
//! Bedrock's touch buttons, which players know: fetched once on the phone from Mojang's own
//! samples repository (bedrock-samples, at the revision `assets/bedrock-touch.json` pins, each
//! file checked by its git blob SHA-1, Mojang's terms kept beside them) and written as the
//! `starling:gui/sprites/touch/<name>` sprites used by Stratum's controls, with
//! player resource packs layered above them. The project carries their
//! names and hashes only. Bedrock has no icon for some of Starling's buttons (pause, chat...):
//! those are its plate with the icon taken off and a glyph of Starling's in its place.
use std::path::{Path, PathBuf};
use std::time::Duration;

const MANIFEST: &str = include_str!("../assets/bedrock-touch.json");

/// The sprites' folder in the resources: their own namespace, which a pack's sprites still win
/// over.
fn folder(resources: &Path) -> PathBuf {
    resources.join("assets/starling/textures/gui/sprites/touch")
}

/// Written last, with the revision: the buttons are all there.
fn marker(resources: &Path) -> PathBuf {
    folder(resources).join(".bedrock")
}

/// The plate's grey, where its icon is taken off.
const FACE: [u8; 4] = [128, 128, 128, 255];
/// Its icon's colour, up (black) and pressed (light grey), as Bedrock draws them.
const INK: [[u8; 4]; 2] = [[0, 0, 0, 255], [230, 230, 230, 255]];

/// Starling's glyphs for the buttons Bedrock has none for, drawn as Bedrock's own: black on the
/// plate, light grey pressed.
const GLYPHS: &[(&str, &str)] = &[
    (
        "custom_gear",
        "..##..##..\n.########.\n###....###\n##..##..##\n##..##..##\n###....###\n.########.\n..##..##..",
    ),
    (
        "pause",
        "###..###\n###..###\n###..###\n###..###\n###..###\n###..###\n###..###\n###..###",
    ),
    ("inventory", "##..##..##\n##..##..##"),
    (
        "chat",
        ".##########.\n#..........#\n#.########.#\n#..........#\n#.######...#\n#..........#\n\
         .###.######.\n...#.#......\n...##.......",
    ),
    (
        "camera",
        "...######...\n.##......##.\n#....##....#\n#...####...#\n#....##....#\n.##......##.\n\
         ...######...",
    ),
    (
        "drop",
        "....##....\n....##....\n....##....\n.########.\n..######..\n...####...\n....##....\n\
         ..........\n##########",
    ),
    (
        "offhand",
        "...#........\n..##........\n.##########.\n..##........\n...#........\n........#...\n\
         ........##..\n.##########.\n........##..\n........#...",
    ),
    (
        "command",
        ".......#\n......##\n.....##.\n....##..\n...##...\n..##....\n.##.....\n##......\n#.......",
    ),
    (
        "debug",
        "######.#####.\n######.######\n##.........##\n##.........##\n#####...####.\n\
         #####...####.\n##.........##\n##.........##\n##.....######\n##.....#####.",
    ),
    (
        "pick",
        "......#......\n....##.##....\n..##.....##..\n##.........##\n#.##.....##.#\n\
         #...##.##...#\n#.....#.....#\n#.....#.....#\n#.....#.....#\n##....#....##\n\
         ..##..#..##..\n....##.##....\n......#......",
    ),
    (
        "players",
        "###.#########\n###.#########\n.............\n###.#########\n###.#########\n\
         .............\n###.#########\n###.#########",
    ),
    // A custom button's icons (settings.rs `CUSTOM_ICONS`), and the plate alone for its canvas.
    ("custom_blank", ""),
    (
        "custom_star",
        "......##......\n......##......\n......##......\n......##......\n##############\n##############\n..##########..\n..##########..\n....######....\n....######....\n..####..####..\n..####..####..\n..##......##..\n..##......##..",
    ),
    (
        "custom_heart",
        "..####..####..\n..####..####..\n##############\n##############\n##############\n##############\n..##########..\n..##########..\n....######....\n....######....\n......##......\n......##......",
    ),
    (
        "custom_shield",
        "##############\n##############\n##############\n##############\n##############\n##############\n##############\n##############\n..##########..\n..##########..\n....######....\n....######....\n......##......\n......##......",
    ),
    (
        "custom_pickaxe",
        "..##########....\n..##########....\n##........####..\n##........####..\n..........######\n..........######\n........####..##\n........####..##\n......####....##\n......####....##\n....####......##\n....####......##\n..####..........\n..####..........\n####............\n####............",
    ),
    (
        "custom_potion",
        "....######....\n....######....\n......##......\n......##......\n......##......\n......##......\n....######....\n....######....\n..##########..\n..##########..\n##############\n##############\n##############\n##############\n..##########..\n..##########..",
    ),
    (
        "custom_apple",
        "........##....\n........##....\n......##......\n......##......\n..####..####..\n..####..####..\n##############\n##############\n##############\n##############\n##############\n##############\n..##########..\n..##########..",
    ),
    (
        "custom_home",
        "......####......\n......####......\n....########....\n....########....\n..############..\n..############..\n################\n################\n..####....####..\n..####....####..\n..####....####..\n..####....####..\n..####....####..\n..####....####..",
    ),
    (
        "custom_flag",
        "####..........\n####..........\n##############\n##############\n##############\n##############\n##############\n##############\n####..........\n####..........\n####..........\n####..........\n####..........\n####..........\n####..........\n####..........",
    ),
    (
        "custom_fire",
        "......##......\n......##......\n....####......\n....####......\n....######..##\n....######..##\n..##########..\n..##########..\n##############\n##############\n######..######\n######..######\n####......####\n####......####\n..##########..\n..##########..",
    ),
    (
        "custom_skull",
        "..##########..\n..##########..\n##############\n##############\n##....##....##\n##....##....##\n##....##....##\n##....##....##\n##############\n##############\n..##########..\n..##########..\n..##..##..##..\n..##..##..##..",
    ),
    (
        "custom_crown",
        "##....####....##\n##....####....##\n##....####....##\n##....####....##\n####..####..####\n####..####..####\n################\n################\n################\n################\n################\n################",
    ),
    (
        "custom_plus",
        "....####....\n....####....\n....####....\n....####....\n############\n############\n############\n############\n....####....\n....####....\n....####....\n....####....",
    ),
    (
        "custom_check",
        "............####\n............####\n..........####..\n..........####..\n####....####....\n####....####....\n..########......\n..########......\n....####........\n....####........\n....##..........\n....##..........",
    ),
    (
        "custom_cross",
        "####....####\n####....####\n..########..\n..########..\n....####....\n....####....\n....####....\n....####....\n..########..\n..########..\n####....####\n####....####",
    ),
    (
        "custom_music",
        "....##########\n....##########\n....##########\n....##########\n....##......##\n....##......##\n....##......##\n....##......##\n######..######\n######..######\n######..######\n######..######\n######..######\n######..######",
    ),
];

/// What the marker holds: the revision and the glyphs drawn, so a glyph added later is drawn on
/// the next start.
fn stamp_key(revision: &str) -> String {
    let names: Vec<&str> = GLYPHS.iter().map(|(name, _)| *name).collect();
    format!("{revision} {}", names.join(","))
}

/// git's name for `bytes`: the SHA-1 of `blob <length>\0` and them.
fn blob_sha1(bytes: &[u8]) -> String {
    use sha1::Digest;
    let mut hash = sha1::Sha1::new();
    hash.update(format!("blob {}\0", bytes.len()).as_bytes());
    hash.update(bytes);
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// An RGBA image as a PNG.
fn png(w: u32, h: u32, pixels: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(pixels).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

/// `plate` (Bedrock's 22x22 sprint button) with its icon taken off: the icon's pixels inside the
/// face become the face's grey.
fn blank(plate: &mut [u8], w: u32, pressed: bool) {
    let ink = INK[usize::from(pressed)];
    for y in 2..20u32 {
        for x in 3..19u32 {
            let i = ((y * w + x) * 4) as usize;
            if plate.get(i..i + 4) == Some(&ink[..]) {
                plate[i..i + 4].copy_from_slice(&FACE);
            }
        }
    }
}

/// `glyph` (rows of `#`) on a blank plate, centred where Bedrock's icons are (a pixel lower when
/// pressed, as the whole button is).
fn stamp(plate: &[u8], w: u32, glyph: &str, pressed: bool) -> Vec<u8> {
    let mut out = plate.to_vec();
    let rows: Vec<&str> = glyph.lines().map(str::trim).collect();
    let gw = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let gh = rows.len() as i32;
    let (ox, oy) = (11 - (gw + 1) / 2, 10 - (gh + 1) / 2 + i32::from(pressed));
    let ink = INK[usize::from(pressed)];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            let (x, y) = (ox + dx as i32, oy + dy as i32);
            if c == b'#' && (0..w as i32).contains(&x) && (0..w as i32).contains(&y) {
                let i = ((y as u32 * w + x as u32) * 4) as usize;
                out[i..i + 4].copy_from_slice(&ink);
            }
        }
    }
    out
}

/// Whether this revision's buttons, and every glyph, are all there.
pub fn present(resources: &Path) -> bool {
    let revision = serde_json::from_str::<serde_json::Value>(MANIFEST)
        .ok()
        .and_then(|m| m["revision"].as_str().map(stamp_key));
    revision.is_some_and(|r| std::fs::read_to_string(marker(resources)).ok() == Some(r))
}

/// Fetches and writes them into `resources` (a prepared resources folder) unless they are there;
/// the number of sprites written.
pub fn fetch(resources: &Path) -> Result<usize, String> {
    if present(resources) {
        return Ok(0);
    }
    if !resources.join("assets/minecraft").is_dir() {
        return Err(format!("{} holds no resources", resources.display()));
    }
    let manifest: serde_json::Value = serde_json::from_str(MANIFEST).map_err(|e| e.to_string())?;
    let (repository, revision) = (
        manifest["repository"].as_str().unwrap_or_default(),
        manifest["revision"].as_str().unwrap_or_default(),
    );
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .build();
    let get = |entry: &serde_json::Value| -> Result<Vec<u8>, String> {
        let source = entry["source"].as_str().unwrap_or_default();
        let url = format!("https://raw.githubusercontent.com/{repository}/{revision}/{source}");
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut agent
                .get(&url)
                .call()
                .map_err(|e| format!("{source}: {e}"))?
                .into_reader(),
            &mut bytes,
        )
        .map_err(|e| format!("{source}: {e}"))?;
        let sha1 = entry["sha1"].as_str().unwrap_or_default();
        if blob_sha1(&bytes) != sha1 {
            return Err(format!("{source}: not the file the manifest names"));
        }
        Ok(bytes)
    };
    let dir = folder(resources);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let write = |name: &str, bytes: &[u8]| {
        let path = dir.join(format!("{name}.png"));
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    };
    let mut written = 0;
    let mut plates = [None, None];
    for entry in manifest["files"].as_array().into_iter().flatten() {
        let name = entry["name"].as_str().unwrap_or_default();
        let bytes = get(entry)?;
        match name {
            "plate" | "plate_pressed" => {
                write(
                    if name == "plate" {
                        "sprint"
                    } else {
                        "sprint_pressed"
                    },
                    &bytes,
                )?;
                let (w, h, mut pixels) =
                    crate::hud::decode_bytes(&bytes).map_err(|e| format!("{name}: {e}"))?;
                let pressed = name == "plate_pressed";
                blank(&mut pixels, w, pressed);
                plates[usize::from(pressed)] = Some((w, h, pixels));
            }
            _ => {
                write(name, &bytes)?;
                written += 1;
            }
        }
    }
    let [Some(up), Some(down)] = plates else {
        return Err("the manifest names no plate".into());
    };
    for (name, glyph) in GLYPHS {
        for (suffix, (w, h, plate), pressed) in [("", &up, false), ("_pressed", &down, true)] {
            write(
                &format!("{name}{suffix}"),
                &png(*w, *h, &stamp(plate, *w, glyph, pressed))?,
            )?;
            written += 1;
        }
    }
    // The terms the textures come with, beside them (out of the sprites' folder).
    let license = &manifest["license"];
    if license.is_object() {
        let text = get(license)?;
        let path = resources.join("assets/starling/bedrock-touch-LICENSE.md");
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    std::fs::write(marker(resources), stamp_key(revision)).map_err(|e| e.to_string())?;
    Ok(written)
}

/// [`fetch`], its outcome in the log (no network: Starling's own buttons, and again next start).
pub fn fetch_logged(resources: &Path) {
    match fetch(resources) {
        Ok(0) => {}
        Ok(n) => log::info!("Horizon: Bedrock's touch buttons fetched from Mojang ({n} sprites)"),
        Err(e) => log::info!("Horizon: Bedrock's touch buttons not fetched: {e}"),
    }
}

/// The prepared-data cache predates downloaded controls and treats missing
/// entries as absent even when the PNG is now on disk. Add these small images
/// as an explicit resource layer before player packs, preserving their priority.
pub fn resources(
    root: &Path,
    base: horizon_assets::resource_source::ResourceSource,
) -> horizon_assets::resource_source::ResourceSource {
    use horizon_assets::resource_source::{Resource, ResourceSource};
    let mut files: std::collections::BTreeMap<_, _> = base
        .list("")
        .into_iter()
        .map(|name| (name.clone(), Resource::file(base.clone(), name)))
        .collect();
    for entry in std::fs::read_dir(folder(root))
        .into_iter()
        .flatten()
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "png") {
            if let Ok(bytes) = std::fs::read(&path) {
                let name = format!(
                    "assets/starling/textures/gui/sprites/touch/{}",
                    entry.file_name().to_string_lossy()
                );
                files.insert(name, Resource::bytes(bytes));
            }
        }
    }
    ResourceSource::layered(files, base)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_blob_hash() {
        assert_eq!(
            blob_sha1(b"hello\n"),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }
}
