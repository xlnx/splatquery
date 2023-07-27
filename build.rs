use std::io::Write;

// Example custom build script.
fn main() {
  // Tell Cargo that if the given file changes, to rerun this build script.
  let gen_i18n = "gen_i18n.py";
  let deps = [gen_i18n, "resources/assets/i18n"];
  for dep in deps {
    println!("cargo:rerun-if-changed={}", dep);
  }

  let out = std::process::Command::new("python3")
    .arg(gen_i18n)
    .output()
    .unwrap();

  if !out.status.success() {
    std::io::stderr().write_all(&out.stderr).unwrap();
    panic!();
  }
}
