use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [input, destination, mode] = args.as_slice() else {
        return ExitCode::FAILURE;
    };
    let obsidian = match mode.to_str() {
        Some("obsidian") => true,
        Some("markdown") => false,
        _ => return ExitCode::FAILURE,
    };
    match export(Path::new(input), Path::new(destination), obsidian) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("export: {:?}", error.kind());
            ExitCode::FAILURE
        }
    }
}

fn export(input: &Path, destination: &Path, obsidian: bool) -> io::Result<()> {
    let note = fs::read_to_string(input.join("content.md"))?;
    let metadata = fs::read(input.join("metadata.json"))?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.starts_with("Capture-") && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .ok_or_else(|| io::Error::from(ErrorKind::InvalidInput))?;
    let image = if obsidian {
        format!("![[{name}/capture.png]]")
    } else {
        "![Capture](capture.png)".into()
    };
    // Keep the image after the note so the YAML front matter stays on the first line.
    let document = format!("{}\n\n{image}\n", note.trim_end());

    // Reserve a fresh folder before writing. Concurrent or repeated exports cannot
    // claim the same destination, even when an existing folder is empty.
    fs::create_dir(destination)?;
    let result = (|| {
        fs::copy(input.join("capture.png"), destination.join("capture.png"))?;
        fs::write(destination.join("metadata.json"), metadata)?;
        fs::write(destination.join("capture.md"), document)
    })();
    if result.is_err() {
        for file in ["capture.png", "metadata.json", "capture.md"] {
            let _ = fs::remove_file(destination.join(file));
        }
        let _ = fs::remove_dir(destination);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn exports_unicode_and_refuses_to_overwrite_or_leave_failed_exports() -> io::Result<()> {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = env::temp_dir().join(format!("screen-translate-{}-{unique}", std::process::id()));
        fs::create_dir(&root)?;
        let result = (|| {
            let input = root.join("input");
            fs::create_dir(&input)?;
            fs::write(input.join("capture.png"), b"fixture")?;
            fs::write(input.join("metadata.json"), b"{}")?;
            fs::write(input.join("content.md"), "---\ntitle: Capture\n---\n# Écran 👋\n")?;
            let markdown = root.join("Capture-1");
            let obsidian = root.join("Capture-2");
            export(&input, &markdown, false)?;
            export(&input, &obsidian, true)?;
            let note = fs::read_to_string(markdown.join("capture.md"))?;
            assert!(note.starts_with("---\ntitle: Capture"));
            assert!(note.contains("# Écran 👋"));
            assert!(note.contains("![Capture](capture.png)"));
            assert!(fs::read_to_string(obsidian.join("capture.md"))?.contains("![[Capture-2/capture.png]]"));
            assert_eq!(export(&input, &markdown, false).unwrap_err().kind(), ErrorKind::AlreadyExists);
            assert_eq!(fs::read(markdown.join("capture.png"))?, b"fixture");
            fs::remove_file(input.join("capture.png"))?;
            let failed = root.join("Capture-3");
            assert!(export(&input, &failed, false).is_err());
            assert!(!failed.exists());
            Ok(())
        })();
        fs::remove_dir_all(root)?;
        result
    }
}
