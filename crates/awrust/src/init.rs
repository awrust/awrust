use std::path::Path;

use tokio::process::Command;

pub async fn run(dir: &Path) {
    if !dir.is_dir() {
        tracing::debug!(path = %dir.display(), "init directory not found, skipping");
        return;
    }

    let mut scripts = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect::<Vec<_>>(),
        Err(e) => {
            tracing::warn!(path = %dir.display(), error = %e, "failed to read init directory");
            return;
        }
    };

    scripts.sort();

    for script in &scripts {
        let name = script.file_name().unwrap_or_default().to_string_lossy();
        tracing::info!(script = %name, "running init script");

        let result = Command::new("/bin/sh").arg(script).status().await;

        match result {
            Ok(status) if status.success() => {
                tracing::info!(script = %name, "init script completed");
            }
            Ok(status) => {
                tracing::error!(script = %name, code = status.code(), "init script failed");
            }
            Err(e) => {
                tracing::error!(script = %name, error = %e, "failed to execute init script");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn skips_nonexistent_directory() {
        run(Path::new("/tmp/awrust-test-nonexistent")).await;
    }

    #[tokio::test]
    async fn runs_scripts_in_sorted_order() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("execution_order");

        for (name, content) in [
            (
                "02_second.sh",
                format!("echo second >> {}", marker.display()),
            ),
            ("01_first.sh", format!("echo first >> {}", marker.display())),
            ("03_third.sh", format!("echo third >> {}", marker.display())),
        ] {
            fs::write(dir.path().join(name), content).unwrap();
        }

        run(dir.path()).await;

        let output = fs::read_to_string(&marker).unwrap();
        assert_eq!(output, "first\nsecond\nthird\n");
    }

    #[tokio::test]
    async fn skips_subdirectories() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("marker");
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(
            dir.path().join("01_script.sh"),
            format!("echo ran >> {}", marker.display()),
        )
        .unwrap();
        fs::write(sub.join("should_not_run.sh"), "exit 1").unwrap();

        run(dir.path()).await;

        let output = fs::read_to_string(&marker).unwrap();
        assert_eq!(output, "ran\n");
    }

    #[tokio::test]
    async fn continues_after_failing_script() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("marker");

        fs::write(dir.path().join("01_fail.sh"), "exit 1").unwrap();
        fs::write(
            dir.path().join("02_ok.sh"),
            format!("echo ok >> {}", marker.display()),
        )
        .unwrap();

        run(dir.path()).await;

        let output = fs::read_to_string(&marker).unwrap();
        assert_eq!(output, "ok\n");
    }

    #[tokio::test]
    async fn empty_directory_is_noop() {
        let dir = TempDir::new().unwrap();
        run(dir.path()).await;
    }
}
