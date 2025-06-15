use super::USBIPD;
use super::USBIPDError::{Error, ErrorCode, USBIPDResult};
use std::io::{self};
use std::process::{Command, Stdio};
use std::thread;
use tracing::{debug, error};

pub struct WslDistribution {
    pub name: String,
    pub running: bool,
    #[allow(dead_code)]
    pub is_default: bool,
    #[allow(dead_code)]
    pub wsl_version: Option<u32>,
}

impl USBIPD {
    #[allow(dead_code)]
    pub fn run_wsl(&self, distribution: Option<&str>, args: Vec<&str>) -> USBIPDResult<String> {
        let mut linux = false;
        let mut cmd = Command::new("wsl.exe");
        if let Some(distribution) = distribution {
            cmd.arg("--distribution").arg(distribution);
            cmd.arg("--user").arg("root");
            cmd.arg("--cd").arg("/");
            cmd.arg("--exec");
            linux = true;
        }
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

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
        debug!("wsl process exit status: {}", status);

        if !status.success() {
            error!("wsl failed with status: {}", status);
            return Err(Error::new(
                ErrorCode::InternalError,
                format!(
                    "wsl failed with status {}: {}",
                    status,
                    if linux {
                        String::from_utf8_lossy(&stderr).to_string()
                    } else {
                        // Convert Vec<u8> to Vec<u16> for from_utf16_lossy
                        let u16_vec: Vec<u16> = stderr
                            .chunks_exact(2)
                            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                            .collect();
                        String::from_utf16_lossy(&u16_vec)
                    },
                ),
            ));
        }
        Ok(if linux {
            String::from_utf8_lossy(&stdout).to_string()
        } else {
            // Convert Vec<u8> to Vec<u16> for from_utf16_lossy
            let u16_vec: Vec<u16> = stdout
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            String::from_utf16_lossy(&u16_vec)
        })
    }

    pub fn get_all_wsl_distribution(&self) -> Result<Vec<WslDistribution>, Error> {
        let output = self
            .run_wsl(None, vec!["--list", "--all", "--verbose"])
            .map_err(|e| {
                error!("Failed to list WSL distribution: {}", e);
                e
            })?;

        let mut vms = vec![];
        let mut lines = output.lines();
        lines.next().ok_or(Error::new(
            ErrorCode::WslDistributionNotFound,
            "No distribution found".to_string(),
        ))?;
        for line in lines {
            // Remove all consecutive internal whitespaces to a single space
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (is_current, s) = if line.starts_with('*') {
                (true, line[1..].trim_start_matches(char::is_whitespace))
            } else {
                (false, line)
            };
            let mut parts = s.split_whitespace();
            let name = parts.next().unwrap_or("").to_string();
            let state = parts.next().unwrap_or("").to_string();
            let version = parts.next().unwrap_or("").parse::<u32>().unwrap_or(0);
            debug!("Parsed WSL distribution: {} {} {}", name, state, version);
            vms.push(WslDistribution {
                name: name,
                running: state.eq_ignore_ascii_case("Running"),
                wsl_version: Some(version),
                is_default: is_current,
            });
        }

        Ok(vms)
    }

    pub fn get_default_or_first_running_wsl_distribution(&self) -> Result<String, Error> {
        let mut distr_name = String::from("");
        let distr_list = self.get_all_wsl_distribution()?;

        for distr in distr_list {
            if distr.running {
                distr_name = distr.name.clone();
                break;
            }
        }

        if distr_name.is_empty() {
            return Err(Error::new(
                ErrorCode::WslDistributionNotFound,
                "No running distribution found".to_string(),
            ));
        }

        Ok(distr_name)
    }
}
