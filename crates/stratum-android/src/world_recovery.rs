//! Non-destructive recovery of worlds left in the old per-launch test folders.
use std::{fs, io, path::Path};
fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    if fs::symlink_metadata(source)?.file_type().is_symlink() {
        return Err(io::Error::other("refusing a symlink in world recovery"));
    }
    if target.exists() && fs::symlink_metadata(target)?.file_type().is_symlink() {
        return Err(io::Error::other("refusing a symlink recovery target"));
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let out = target.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &out)?;
        } else if kind.is_file() {
            if out.exists() && !fs::symlink_metadata(&out)?.is_file() {
                return Err(io::Error::other("invalid world recovery destination"));
            }
            fs::copy(entry.path(), out)?;
        } else {
            return Err(io::Error::other("unsupported entry in world recovery"));
        }
    }
    Ok(())
}
pub fn recover(private: &Path, game: &Path) -> io::Result<usize> {
    let marker = game.join(".legacy-worlds-recovered-v1");
    if marker.exists() {
        return Ok(0);
    }
    let runs = private.join("test-runs");
    if !runs.is_dir() {
        return Ok(0);
    }
    let saves = game.join("saves");
    let staging = private.join("world-recovery-staging");
    fs::create_dir_all(&saves)?;
    fs::create_dir_all(&staging)?;
    let mut count = 0;
    for run in fs::read_dir(runs)? {
        let run = run?;
        if !run.file_type()?.is_dir() {
            continue;
        }
        let source = run.path().join("game/saves");
        if !source.is_dir() {
            continue;
        }
        for world in fs::read_dir(source)? {
            let world = world?;
            if !world.file_type()?.is_dir() || !world.path().join("level.dat").is_file() {
                continue;
            }
            let name = format!(
                "Recovered-{}-{}",
                run.file_name().to_string_lossy(),
                world.file_name().to_string_lossy()
            );
            let target = saves.join(&name);
            // A previous attempt may have recovered some worlds already. Never replace one.
            if target.exists() {
                continue;
            }
            let temp = staging.join(&name);
            copy_tree(&world.path(), &temp)?;
            fs::rename(temp, target)?;
            count += 1;
        }
    }
    fs::write(marker, b"Original test-session worlds retained.\n")?;
    Ok(count)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(name: &str) -> std::path::PathBuf {
        let p =
            std::env::temp_dir().join(format!("stratum-recovery-{}-{name}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        p
    }
    #[test]
    fn recovers_same_named_worlds_without_overwriting_or_deleting() {
        let p = fixture("copies");
        let game = p.join("play/game");
        for (session, bytes) in [
            ("run.a", b"first".as_slice()),
            ("run.b", b"second".as_slice()),
        ] {
            let world = p.join(format!("test-runs/{session}/game/saves/New World"));
            fs::create_dir_all(world.join("region")).unwrap();
            fs::write(world.join("level.dat"), bytes).unwrap();
            fs::write(world.join("region/r.0.0.mca"), bytes).unwrap();
        }
        fs::create_dir_all(game.join("saves/New World")).unwrap();
        fs::write(game.join("saves/New World/level.dat"), b"existing").unwrap();
        assert_eq!(recover(&p, &game).unwrap(), 2);
        assert_eq!(
            fs::read(game.join("saves/Recovered-run.a-New World/level.dat")).unwrap(),
            b"first"
        );
        assert_eq!(
            fs::read(game.join("saves/Recovered-run.b-New World/region/r.0.0.mca")).unwrap(),
            b"second"
        );
        assert_eq!(
            fs::read(game.join("saves/New World/level.dat")).unwrap(),
            b"existing"
        );
        assert!(
            p.join("test-runs/run.a/game/saves/New World/level.dat")
                .exists()
        );
        assert_eq!(recover(&p, &game).unwrap(), 0);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn skips_non_world_folders_and_can_resume_after_partial_import() {
        let p = fixture("resume");
        let game = p.join("play/game");
        fs::create_dir_all(p.join("test-runs/run.a/game/saves/not-a-world")).unwrap();
        let src = p.join("test-runs/run.a/game/saves/World");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("level.dat"), b"source").unwrap();
        fs::create_dir_all(game.join("saves/Recovered-run.a-World")).unwrap();
        fs::write(
            game.join("saves/Recovered-run.a-World/level.dat"),
            b"played since import",
        )
        .unwrap();
        assert_eq!(recover(&p, &game).unwrap(), 0);
        assert_eq!(
            fs::read(game.join("saves/Recovered-run.a-World/level.dat")).unwrap(),
            b"played since import"
        );
        assert!(!game.join("saves/Recovered-run.a-not-a-world").exists());
        fs::remove_dir_all(p).unwrap();
    }
}
