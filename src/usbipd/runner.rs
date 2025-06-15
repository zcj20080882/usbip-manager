use super::USBIPDError;

use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncRead};
use tokio::sync::mpsc;
use std::io::{self};
use std::process::Stdio;
use windows_elevate::{check_elevated};
use tracing::{debug, error};

async fn process_output(
    reader: BufReader<impl AsyncRead + Unpin>,
    tx: mpsc::Sender<String>,
) -> io::Result<()> {
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if tx.send(line).await.is_err() {
            // Receiver has been closed
            break;
        }
    }

    Ok(())
}


pub async fn run_powershell_script(script: &str) -> Result<String, USBIPDError> {
    let args = format!("$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new();{}", script);
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(args);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut output = cmd
        .spawn()
        .map_err(|_| USBIPDError::PowerShellError)?;

    let stdout = output.stdout.take().ok_or_else(|| {
        USBIPDError::PowerShellError
    })?;
    let stderr = output.stderr.take().ok_or_else(|| {
        USBIPDError::PowerShellError
    })?;

    let (stdout_tx, mut stdout_rx) = mpsc::channel(100);
    let (stderr_tx, mut stderr_rx) = mpsc::channel(100);

    // 异步处理 stdout
    // 异步处理 stdout
    tokio::spawn(async move {
        match process_output(BufReader::new(stdout), stdout_tx).await {
            Err(e) => {
                error!("Error processing stdout: {}", e);
            }
            _ => (),
        }
    });

    // 异步处理 stderr
    tokio::spawn(async move {
        if let Err(e) = process_output(BufReader::new(stderr), stderr_tx).await {
            error!("Error processing stderr: {}", e);
        }
    });

    let status = output.wait().await.map_err(|_| USBIPDError::PowerShellError)?;
    if !status.success() {
        let mut result = String::new();
        while let Some(line) = stderr_rx.recv().await {
            result.push_str(&line);
            result.push('\n');
        }
        error!("{}\n", result);
        return Err(USBIPDError::PowerShellError);
    }
    let mut result = String::new();
    while let Some(line) = stdout_rx.recv().await {
        result.push_str(&line);
        result.push('\n');
    }
    Ok(result)
}

pub async fn run_usbipd_command(command: Vec<&str>, admin: bool) -> Result<String, USBIPDError> {

    let output;
    let is_admin = check_elevated().unwrap_or(false);
    if admin && !is_admin {
        let mut cmd: Vec<&str> = Vec::new();
        let args = format!("'{}'", command.join(" "));
        cmd.push("Start-Process");
        cmd.push("usbipd.exe");
        cmd.push("-ArgumentList");
        cmd.push(args.as_str());
        cmd.push("-WindowStyle");
        cmd.push("Hidden");
        cmd.push("-Verb");
        cmd.push("RunAs");

        output = Command::new("powershell.exe")
        .args(&cmd)
        .output()
        .await
        .map_err(|_| USBIPDError::InternalError)?;
        debug!("Elevating privileges to run usbipd command.\n");

    }
    else{
        output = Command::new("usbipd")
        .args(&command)
        .output()
        .await
        .map_err(|_| USBIPDError::InternalError)?;
    }

    if !output.status.success() {
        print!("usbipd command failed with status: {}\n", output.status);
        let combined = format!(
            "STDOUT:\n{}\n\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        debug!("{}", combined);
        return Err(USBIPDError::InternalError);
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
