use super::USBIPD;
use super::USBIPDError::{Error, ErrorCode, USBIPDResult};

use std::io::{self};
use std::process::{Command, Stdio};
use std::thread;
use tracing::{debug, error};
use windows_elevate::check_elevated;

impl USBIPD {
    pub fn run_program(&self, program: &str, args: Vec<&str>) -> USBIPDResult<String> {
        let mut cmd = Command::new(program);
        cmd.args(args.iter())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                // 通过错误码判断程序是否存在
                if let Some(2) = e.raw_os_error() {
                    return Err(Error::new(
                        ErrorCode::CommandNotFound,
                        "Command not found".to_string(),
                    ));
                }
                return Err(Error::new(ErrorCode::InternalError, e.to_string()));
            }
        };
        let mut out = child.stdout.take().unwrap();
        let mut err = child.stderr.take().unwrap();

        let out_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            io::copy(&mut out, &mut buf).ok();
            buf
        });
        let err_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            io::copy(&mut err, &mut buf).ok();
            buf
        });

        let stdout = out_handle.join().unwrap();
        let stderr = err_handle.join().unwrap();
        let status = child.wait().map_err(|e| {
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to wait for child process: {}", e),
            )
        })?;
        debug!("{} exit status: {}", program, status);

        if !status.success() {
            error!("{} failed with status: {}", program, status);
            return Err(Error::new(
                ErrorCode::InternalError,
                format!(
                    "{} failed with status {}: {}",
                    program,
                    status,
                    String::from_utf8_lossy(&stderr)
                ),
            ));
        }
        Ok(String::from_utf8_lossy(&stdout).to_string())
    }

    pub fn run_usbipd_command(&self, command: Vec<&str>, admin: bool) -> USBIPDResult<String> {
        let output;
        let is_admin = check_elevated().unwrap_or(false);
        let usbipd_path = self
            .path()
            .map_err(|_| Error::new(ErrorCode::UsbIPDNotFound, "usbipd path not found."))?;
        let program;
        let args;
        let cmd_args = format!("\"{}\"", command.clone().join(" "));
        let usbipd_path_quoted = format!("\"{}\"", usbipd_path);

        if admin && !is_admin {
            program = "powershell.exe";

            args = vec![
                "-NoProfile",
                "Start-Process",
                &usbipd_path_quoted,
                "-ArgumentList",
                cmd_args.as_str(),
                "-WindowStyle",
                "Hidden",
                "-Verb",
                "RunAs",
            ];
            debug!(
                "Elevating privileges to run usbipd command: {} {}.\n",
                program,
                args.join(" ")
            );
        } else {
            program = &usbipd_path;
            args = command.clone();
            debug!("Running usbipd command: {} {}\n", program, args.join(" "));
        }

        match self.run_program(program, args) {
            Ok(output_str) => {
                output = output_str;
                Ok(output)
            }
            Err(e) => {
                error!("Failed to run usbipd command: {}", e);
                Err(e)
            }
        }
    }
}
