use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn missing_src_file() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("secp")?;

  cmd.arg("someuser").arg("file/that/does/not/exist").arg("somedest");
  cmd.assert()
    .failure()
    .stderr(predicate::str::contains("not able to read file"));

  Ok(())
}
