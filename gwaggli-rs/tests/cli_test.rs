use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;
#[test]
fn test_cli_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("gwaggli-rs")?;

    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: gwaggli-rs"))
        .stdout(predicate::str::contains("Arguments:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Options:"));

    Ok(())
}

#[test]
fn test_cli_transcribe() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("gwaggli-rs")?;

    cmd.arg("transcribe")
        .arg("--input")
        .arg("./test_data/audio/riff_wave/pcm_s16le_16k_mono.wav")
        .arg("--quality")
        .arg("low");

    cmd.assert().success().stdout(predicate::str::contains(
        "Plans are well underway for races to Mars and the Moon in 1992 by solar sales.",
    ));

    Ok(())
}
