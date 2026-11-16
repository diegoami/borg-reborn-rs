use angband_recorder::{validate_recording, verify};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Serialize, Deserialize)]
struct Launch {
    executable: PathBuf,
    sha256: String,
    arguments: Vec<String>,
    cwd: PathBuf,
}
fn digest(path: &Path) -> io::Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn replay_arguments(arguments: &[String], profile_root: &Path) -> io::Result<Vec<String>> {
    // Vanilla resolves -u against the save directory that is current while
    // arguments are parsed, so the private directories must precede every
    // remaining engine argument; appended overrides would leave a -u before
    // them frozen onto the original or default save path. Original overrides
    // of the same directories are removed wherever they appear.
    let split = arguments
        .iter()
        .position(|a| a == "--")
        .unwrap_or(arguments.len());
    let mut body: Vec<String> = arguments[..split].to_vec();
    let tail: Vec<String> = arguments[split..].to_vec();
    let mut directories = Vec::new();
    for key in ["user", "save", "panic", "archive", "scores"] {
        let directory = profile_root.join(key);
        fs::create_dir_all(&directory)?;
        directories.push(format!("-d{key}={}", directory.display()));
        body.retain(|arg| !arg.starts_with(&format!("-d{key}=")));
    }
    let mut result = directories;
    result.extend(body);
    result.extend(tail);
    Ok(result)
}

fn run(
    launch: &Launch,
    output: &Path,
    original: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if digest(&launch.executable)? != launch.sha256 {
        return Err("host executable differs from the recorded build".into());
    }
    fs::create_dir_all(output)?;
    if fs::read_dir(output)?.next().is_some() {
        return Err("output directory must be empty".into());
    }
    let mut command = Command::new(&launch.executable);
    for name in [
        "BORG_RECORD_DIR",
        "BORG_REPLAY_DIR",
        "ANGBAND_RECORD_DIR",
        "ANGBAND_REPLAY_DIR",
    ] {
        command.env_remove(name);
    }
    let mut arguments = launch.arguments.clone();
    if let Some(original) = original {
        validate_recording(original)?;
        // Replays read frozen file bytes, but all game writes use a new private
        // profile. Never overwrite the profile named by the original recipe.
        arguments = replay_arguments(&arguments, &output.join("profile"))?;
    }
    command
        .current_dir(&launch.cwd)
        .args(&arguments)
        .env("ANGBAND_RECORD_DIR", output);
    if let Some(original) = original {
        command.env("ANGBAND_REPLAY_DIR", original);
    }
    let status = command.status()?;
    if !status.success() {
        return Err(format!("host exited {status}").into());
    }
    if !output.join("complete.json").is_file() {
        return Err("host did not complete a recording (is the recorder enabled?)".into());
    }
    Ok(())
}
fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let usage = "Usage: angband-recorder record OUTPUT -- EXECUTABLE [ARG...]\n       angband-recorder replay --verify SOURCE OUTPUT\n       angband-recorder verify SOURCE REPLAY\n       angband-recorder list SOURCE\n       angband-recorder view SOURCE [FRAME]";
    match args.first().map(String::as_str) {
        Some("record") if args.len() >= 4 && args[2] == "--" => {
            let output = absolute(&args[1])?;
            let launch = Launch {
                executable: fs::canonicalize(&args[3])?,
                sha256: digest(Path::new(&args[3]))?,
                arguments: args[4..].to_vec(),
                cwd: env::current_dir()?,
            };
            run(&launch, &output, None)?;
            serde_json::to_writer_pretty(File::create(output.join("launch.json"))?, &launch)?;
            println!("Recorded {}", output.display());
        }
        Some("replay") if args.len() == 4 && args[1] == "--verify" => {
            let source = fs::canonicalize(&args[2])?;
            let output = absolute(&args[3])?;
            let launch: Launch = serde_json::from_reader(File::open(source.join("launch.json"))?)?;
            run(&launch, &output, Some(&source))?;
            verify(&source, &output)?;
            println!("VERIFIED: inputs, file bytes, messages, frames and completion receipt");
        }
        Some("verify") if args.len() == 3 => {
            verify(Path::new(&args[1]), Path::new(&args[2]))?;
            println!("VERIFIED: supplied streams match (no re-execution by this command)");
        }
        Some("list") if args.len() == 2 => {
            println!(
                "{}",
                fs::read_to_string(Path::new(&args[1]).join("recording.json"))?
            );
            println!(
                "{}",
                fs::read_to_string(Path::new(&args[1]).join("complete.json"))?
            );
        }
        Some("view") if args.len() == 2 || args.len() == 3 => {
            let wanted: u64 = args.get(2).map_or(Ok(1), |v| v.parse())?;
            let mut screen = Vec::new();
            let mut dimensions = (0, 0);
            for line in
                BufReader::new(File::open(Path::new(&args[1]).join("frames.jsonl"))?).lines()
            {
                let frame: serde_json::Value = serde_json::from_str(&line?)?;
                let w = frame["width"].as_u64().ok_or("missing width")? as usize;
                let h = frame["height"].as_u64().ok_or("missing height")? as usize;
                if w == 0 || h == 0 || w > 255 || h > 255 {
                    return Err("invalid frame geometry".into());
                }
                if dimensions != (w, h) {
                    screen = vec![' '; w * h];
                    dimensions = (w, h);
                }
                for cell in frame["cells"].as_array().ok_or("missing cells")? {
                    let at = cell[0].as_u64().ok_or("invalid cell offset")? as usize;
                    let code = cell[1].as_u64().ok_or("invalid character")? as u32;
                    *screen.get_mut(at).ok_or("cell outside screen")? = char::from_u32(code)
                        .filter(|c| !c.is_control())
                        .unwrap_or(' ');
                }
                if frame["index"].as_u64() == Some(wanted) {
                    println!(
                        "Frame {wanted}, turn {}, depth {}, HP {}/{}",
                        frame["turn"], frame["depth"], frame["hp"], frame["max_hp"]
                    );
                    for row in screen.chunks(w) {
                        println!("{}", row.iter().collect::<String>());
                    }
                    return Ok(());
                }
            }
            return Err("frame not found".into());
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}
fn absolute(path: &str) -> io::Result<PathBuf> {
    let path = Path::new(path);
    if path.is_absolute() {
        Ok(path.into())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}
fn main() -> ExitCode {
    match main_result() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_f20_private_directories_precede_every_engine_argument() {
        let root = std::env::temp_dir().join("recorder-f20-test");
        let profile = &root;
        // The escape shape: -u is parsed while ANGBAND_DIR_SAVE still holds
        // the default, so private directories must come first.
        let args = [
            "/game/angband".into(),
            "-dsave=/original".into(),
            "-uRecorderTest".into(),
            "--".into(),
            "-x".into(),
        ];
        let result = replay_arguments(&args, profile).unwrap();
        let expected_prefix: Vec<String> = ["user", "save", "panic", "archive", "scores"]
            .iter()
            .map(|key| format!("-d{key}={}", profile.join(key).display()))
            .collect();
        assert_eq!(&result[..5], &expected_prefix);
        assert_eq!(
            &result[5..],
            &["/game/angband", "-uRecorderTest", "--", "-x"]
        );
        assert!(result.iter().all(|arg| !arg.contains("/original")));
    }

    #[test]
    fn review_f20_original_directory_overrides_are_removed_before_the_tail() {
        let profile = std::env::temp_dir().join("recorder-f20-tail");
        let args = [
            "-uName".into(),
            "-duser=/old".into(),
            "--".into(),
            "-dsave=/after".into(),
        ];
        let result = replay_arguments(&args, &profile).unwrap();
        // The private directories lead; original overrides in the parsed body
        // are stripped; the unparsed tail after "--" is passed through
        // verbatim (the engine never interprets options there).
        assert_eq!(&result[5..], &["-uName", "--", "-dsave=/after"]);
        assert!(result.iter().take(5).enumerate().all(|(i, arg)| {
            let keys = ["user", "save", "panic", "archive", "scores"];
            arg == &format!("-d{}={}", keys[i], profile.join(keys[i]).display())
        }));
    }
}
