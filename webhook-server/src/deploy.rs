use std::env;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

use crate::config::ResolvedSshConfig;

pub const MAX_RETRIES: u32 = 2;
pub const RETRY_DELAY_SECS: u64 = 300;

pub struct VenvSettings<'a> {
    pub use_venv: bool,
    pub venv_dir: &'a str,
    pub python_bin: &'a str,
    pub upgrade_pip: bool,
}

pub async fn run_deployment(
    source_dir: &str,
    deploy_dir: &str,
    repo_url: &str,
    repo_name: &str,
    branch_ref: &str,
    venv: &VenvSettings<'_>,
    ssh: Option<&ResolvedSshConfig>,
) -> Result<String, String> {
    let mut last_error: Option<String> = None;

    for attempt in 0..=MAX_RETRIES {
        info!(
            "Deployment attempt {} of {} for {}",
            attempt + 1,
            MAX_RETRIES + 1,
            repo_name
        );

        match run_deployment_once(source_dir, deploy_dir, repo_url, repo_name, branch_ref, venv)
            .await
        {
            Ok(output) => {
                info!(
                    "Deployment succeeded on attempt {} for {}",
                    attempt + 1,
                    repo_name
                );

                if let Some(ssh_config) = ssh {
                    if let Err(e) = ssh_deploy(deploy_dir, repo_name, repo_name, ssh_config).await {
                        error!("SSH deployment failed for {}: {}", repo_name, e);
                        return Err(format!("Local deploy OK but SSH deploy failed: {}", e));
                    }
                }

                return Ok(output);
            }
            Err(e) => {
                error!(
                    "Deployment attempt {} failed for {}: {}",
                    attempt + 1,
                    repo_name,
                    e
                );
                last_error = Some(e);

                if attempt < MAX_RETRIES {
                    info!(
                        "Retrying deployment for {} in {} seconds...",
                        repo_name, RETRY_DELAY_SECS
                    );
                    time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Deployment failed after all retries".to_string()))
}

pub async fn git_sync(
    source_dir: &str,
    repo_url: &str,
    repo_name: &str,
    branch_ref: &str,
) -> Result<String, String> {
    let repo_path = format!("{}/{}", source_dir.trim_end_matches('/'), repo_name);
    let branch_name = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let repo_exists = tokio::fs::try_exists(&repo_path).await.unwrap_or(false);

    if repo_exists {
        info!(
            "Repository {} exists, syncing to remote branch {}...",
            repo_name, branch_name
        );
        run_git_command(&repo_path, &["fetch", "origin", branch_name]).await?;
        run_git_command(
            &repo_path,
            &["checkout", "-B", branch_name, &format!("origin/{}", branch_name)],
        )
        .await?;
        run_git_command(
            &repo_path,
            &["reset", "--hard", &format!("origin/{}", branch_name)],
        )
        .await?;
        run_git_command(&repo_path, &["clean", "-fd"]).await?;
        info!(
            "Repository {} synced to origin/{} successfully",
            repo_name, branch_name
        );
    } else {
        info!("Cloning repository {}...", repo_url);
        tokio::fs::create_dir_all(source_dir)
            .await
            .map_err(|e| format!("Failed to create source directory: {}", e))?;

        let mut child = Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(branch_name)
            .arg(repo_url)
            .arg(&repo_path)
            .env("GIT_SSH_COMMAND", GIT_SSH_OPTS)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn git clone: {}", e))?;

        let result = tokio::time::timeout(
            Duration::from_secs(GIT_TIMEOUT_SECS),
            child.wait(),
        )
        .await;

        match result {
            Ok(Ok(status)) => {
                if !status.success() {
                    let stderr = if let Some(mut se) = child.stderr.take() {
                        let mut buf = String::new();
                        let _ = tokio::io::AsyncReadExt::read_to_string(&mut se, &mut buf).await;
                        buf
                    } else {
                        String::new()
                    };
                    return Err(format!("git clone failed: {}", stderr));
                }
            }
            Ok(Err(e)) => return Err(format!("Failed waiting for git clone: {}", e)),
            Err(_) => {
                let _ = child.kill().await;
                return Err(format!("git clone timed out after {}s", GIT_TIMEOUT_SECS));
            }
        }

        info!("git clone completed for {}", repo_name);
    }

    Ok(format!("Synced {} to branch {}", repo_name, branch_name))
}

/// PLACEHOLDER_BUILD_AND_DEPLOY

pub async fn build_docs(
    source_dir: &str,
    deploy_dir: &str,
    repo_name: &str,
    venv: &VenvSettings<'_>,
) -> Result<String, String> {
    let repo_path = format!("{}/{}", source_dir.trim_end_matches('/'), repo_name);
    let docs_path = format!("{}/docs", repo_path);

    if !tokio::fs::try_exists(&docs_path).await.unwrap_or(false) {
        return Err(format!("docs folder not found in {}", repo_name));
    }

    let venv_path_opt = if venv.use_venv {
        let path = ensure_venv(&docs_path, venv.venv_dir, venv.python_bin).await?;
        ensure_pip(&path, venv.upgrade_pip).await?;
        Some(path)
    } else {
        None
    };

    // Install pip dependencies
    let requirements_path = format!("{}/requirements.txt", docs_path);
    if tokio::fs::try_exists(&requirements_path).await.unwrap_or(false) {
        let (pip_program, pip_first_args): (String, Vec<String>) = match &venv_path_opt {
            Some(path) => (
                format!("{}/bin/python", path),
                vec!["-m".to_string(), "pip".to_string()],
            ),
            None => ("pip".to_string(), Vec::new()),
        };

        let mut cmd = Command::new(&pip_program);
        for a in &pip_first_args {
            cmd.arg(a);
        }
        let output = cmd
            .arg("install")
            .arg("-r")
            .arg(&requirements_path)
            .current_dir(&docs_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run pip install: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "pip install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    let path_env = venv_path_opt.as_deref().map(venv_path_env);

    // chmod scripts
    let _ = Command::new("find")
        .arg(".")
        .arg("(")
        .arg("-path")
        .arg("./scripts/*")
        .arg("-o")
        .arg("-name")
        .arg("*.sh")
        .arg(")")
        .arg("-type")
        .arg("f")
        .arg("-exec")
        .arg("chmod")
        .arg("+x")
        .arg("{}")
        .arg("+")
        .current_dir(&docs_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    // make clean
    let mut make_clean = Command::new("make");
    make_clean.arg("clean").current_dir(&docs_path);
    if let Some(ref pe) = path_env {
        make_clean.env("PATH", pe);
    }
    let _ = make_clean
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    // make dist
    let mut make_dist = Command::new("make");
    make_dist.arg("dist").current_dir(&docs_path);
    if let Some(ref pe) = path_env {
        make_dist.env("PATH", pe);
    }
    run_logged_command(make_dist, "make dist").await?;

    // Copy dist to deploy directory
    let dist_path = format!("{}/dist", docs_path);
    let target_dir = format!("{}/{}", deploy_dir.trim_end_matches('/'), repo_name);

    info!("Removing old deploy directory: {}", target_dir);
    tokio::fs::remove_dir_all(&target_dir).await.unwrap_or(());
    info!("Creating deploy directory: {}", target_dir);
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create deploy directory: {}", e))?;
    info!("Copying dist ({}) to {}", dist_path, target_dir);
    let src = dist_path.clone();
    let dst = target_dir.clone();
    tokio::task::spawn_blocking(move || copy_directory(&src, &dst))
        .await
        .map_err(|e| format!("Copy task panicked: {}", e))?
        .map_err(|e| format!("Failed to copy dist: {}", e))?;
    info!("Copy completed successfully");

    info!("Build and local deploy completed for {}", repo_name);
    Ok(format!("Built and deployed {} to {}", repo_name, target_dir))
}

/// PLACEHOLDER_SSH_DEPLOY

pub async fn ssh_deploy(
    deploy_dir: &str,
    repo_name: &str,
    remote_name: &str,
    ssh: &ResolvedSshConfig,
) -> Result<String, String> {
    let local_path = format!("{}/{}/", deploy_dir.trim_end_matches('/'), repo_name);
    let remote_dir = format!(
        "{}/{}/",
        ssh.remote_path.trim_end_matches('/'),
        remote_name
    );
    let remote_dest = format!(
        "{}@{}:{}",
        ssh.user,
        ssh.host,
        remote_dir
    );

    info!("SSH deploying {} to {}...", repo_name, remote_dest);

    let ssh_cmd = format!(
        "ssh -p {} -o StrictHostKeyChecking=no",
        ssh.port
    );

    // Remove remote directory first to ensure clean deploy
    let rm_cmd = format!("rm -rf {}", remote_dir);
    let rm_output = Command::new("sshpass")
        .arg("-p")
        .arg(&ssh.password)
        .arg("ssh")
        .arg("-p")
        .arg(ssh.port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg(format!("{}@{}", ssh.user, ssh.host))
        .arg(&rm_cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to remove remote directory: {}", e))?;

    if !rm_output.status.success() {
        warn!(
            "Remote rm failed (may not exist): {}",
            String::from_utf8_lossy(&rm_output.stderr)
        );
    }

    let output = Command::new("sshpass")
        .arg("-p")
        .arg(&ssh.password)
        .arg("rsync")
        .arg("-avz")
        .arg("--delete")
        .arg("--force")
        .arg("--mkpath")
        .arg("-e")
        .arg(&ssh_cmd)
        .arg(&local_path)
        .arg(&remote_dest)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run sshpass/rsync: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rsync failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    info!("SSH deployment completed for {}", repo_name);
    Ok(format!("SSH deployed {} to {}", repo_name, remote_dest))
}

async fn run_deployment_once(
    source_dir: &str,
    deploy_dir: &str,
    repo_url: &str,
    repo_name: &str,
    branch_ref: &str,
    venv: &VenvSettings<'_>,
) -> Result<String, String> {
    git_sync(source_dir, repo_url, repo_name, branch_ref).await?;
    build_docs(source_dir, deploy_dir, repo_name, venv).await
}

const GIT_TIMEOUT_SECS: u64 = 120;

const GIT_SSH_OPTS: &str = "ssh -o StrictHostKeyChecking=no -o BatchMode=yes -o ConnectTimeout=15 -o ServerAliveInterval=10 -o ServerAliveCountMax=3";

async fn run_git_command(repo_path: &str, args: &[&str]) -> Result<(), String> {
    let label = format!("git {}", args.first().unwrap_or(&""));
    info!("Running {} in {}", label, repo_path);

    let mut child = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("GIT_SSH_COMMAND", GIT_SSH_OPTS)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", label, e))?;

    let result = tokio::time::timeout(
        Duration::from_secs(GIT_TIMEOUT_SECS),
        child.wait(),
    )
    .await;

    match result {
        Ok(Ok(status)) => {
            if !status.success() {
                let stderr = if let Some(mut se) = child.stderr.take() {
                    let mut buf = String::new();
                    let _ = tokio::io::AsyncReadExt::read_to_string(&mut se, &mut buf).await;
                    buf
                } else {
                    String::new()
                };
                return Err(format!("{} failed: {}", label, stderr));
            }
            Ok(())
        }
        Ok(Err(e)) => Err(format!("Failed waiting for {}: {}", label, e)),
        Err(_) => {
            let _ = child.kill().await;
            Err(format!("{} timed out after {}s", label, GIT_TIMEOUT_SECS))
        }
    }
}

/// PLACEHOLDER_HELPERS

async fn ensure_venv(docs_path: &str, venv_dir: &str, python_bin: &str) -> Result<String, String> {
    let venv_path = format!("{}/{}", docs_path.trim_end_matches('/'), venv_dir);

    let python_in_venv = format!("{}/bin/python", venv_path);
    let python3_in_venv = format!("{}/bin/python3", venv_path);
    let usable = tokio::fs::metadata(&python_in_venv).await.is_ok()
        && tokio::fs::metadata(&python3_in_venv).await.is_ok();

    if usable {
        return Ok(venv_path);
    }

    if tokio::fs::try_exists(&venv_path).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(&venv_path)
            .await
            .map_err(|e| format!("Failed to remove incomplete venv: {}", e))?;
    }

    let output = Command::new(python_bin)
        .arg("-m")
        .arg("venv")
        .arg(&venv_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run `{} -m venv`: {}", python_bin, e))?;

    if output.status.success() {
        return Ok(venv_path);
    }

    if tokio::fs::try_exists(&venv_path).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(&venv_path).await.map_err(|e| {
            format!("Failed to remove failed venv: {}", e)
        })?;
    }

    let output = Command::new(python_bin)
        .arg("-m")
        .arg("venv")
        .arg("--without-pip")
        .arg(&venv_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to create venv --without-pip: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create venv: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(venv_path)
}

/// PLACEHOLDER_ENSURE_PIP

async fn ensure_pip(venv_path: &str, upgrade: bool) -> Result<(), String> {
    let pip_bin = format!("{}/bin/pip", venv_path);
    let python_bin = format!("{}/bin/python", venv_path);

    if !tokio::fs::try_exists(&pip_bin).await.unwrap_or(false) {
        let output = Command::new(&python_bin)
            .arg("-m")
            .arg("ensurepip")
            .arg("--upgrade")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run ensurepip: {}", e))?;

        if !output.status.success() {
            let get_pip_path = format!("{}/get-pip.py", venv_path);
            let resp = reqwest::get("https://bootstrap.pypa.io/get-pip.py")
                .await
                .map_err(|e| format!("Failed to download get-pip.py: {}", e))?;
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Failed to read get-pip.py: {}", e))?;
            tokio::fs::write(&get_pip_path, &bytes)
                .await
                .map_err(|e| format!("Failed to write get-pip.py: {}", e))?;

            let output = Command::new(&python_bin)
                .arg(&get_pip_path)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| format!("Failed to run get-pip.py: {}", e))?;

            let _ = tokio::fs::remove_file(&get_pip_path).await;

            if !output.status.success() {
                return Err(format!(
                    "get-pip.py failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }

    if upgrade {
        let output = Command::new(&python_bin)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--upgrade")
            .arg("pip")
            .arg("setuptools")
            .arg("wheel")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to upgrade pip: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "pip upgrade failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}

fn venv_path_env(venv_path: &str) -> String {
    let venv_bin = format!("{}/bin", venv_path.trim_end_matches('/'));
    match env::var("PATH") {
        Ok(existing) if !existing.is_empty() => format!("{}:{}", venv_bin, existing),
        _ => venv_bin,
    }
}

async fn run_logged_command(mut command: Command, label: &str) -> Result<(), String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", label, e))?;

    let stdout = child.stdout.take().ok_or_else(|| format!("No stdout for {}", label))?;
    let stderr = child.stderr.take().ok_or_else(|| format!("No stderr for {}", label))?;

    let stdout_label = label.to_string();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await.unwrap_or(None) {
            info!("[{} stdout] {}", stdout_label, line);
        }
    });

    let stderr_label = label.to_string();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await.unwrap_or(None) {
            warn!("[{} stderr] {}", stderr_label, line);
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed waiting for {}: {}", label, e))?;

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with status {}", label, status))
    }
}

pub fn copy_directory(src: &str, dst: &str) -> std::io::Result<()> {
    use std::fs;
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = std::path::Path::new(dst).join(entry.file_name());
        if entry.metadata()?.is_dir() {
            copy_directory(src_path.to_str().unwrap(), dst_path.to_str().unwrap())?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
