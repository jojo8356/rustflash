use crate::device::detect::DeviceInfo;
use crate::platform::DeviceEnumerator;
use anyhow::{Context, Result};
use serde_json::Value;
use std::process::Command;

const POWERSHELL_CANDIDATES: [&str; 2] = ["powershell", "pwsh"];

/// Structure publique `WindowsEnumerator`
pub struct WindowsEnumerator;

impl DeviceEnumerator for WindowsEnumerator {
    fn list_devices(&self, include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
        let output = run_powershell(
            "Get-CimInstance Win32_DiskDrive | Select-Object DeviceID,Model,Size,MediaType,InterfaceType,Index,PNPDeviceID | ConvertTo-Json -Compress",
        )?;

        let mut devices = Vec::new();
        let entries = parse_json_array(&output)?;

        for entry in entries {
            let device_path = entry
                .get("DeviceID")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            if device_path.is_empty() {
                continue;
            }

            let path_lower = device_path.to_ascii_lowercase();
            if !path_lower.starts_with(r"\\.\physicaldrive") {
                continue;
            }

            let model = entry.get("Model").and_then(Value::as_str).map(ToString::to_string);
            let size = entry
                .get("Size")
                .and_then(Value::as_u64)
                .or_else(|| entry.get("Size").and_then(Value::as_str).and_then(|v| v.parse().ok()))
                .unwrap_or(0);
            let media_type = entry.get("MediaType").and_then(Value::as_str).unwrap_or("");
            let interface_type = entry.get("InterfaceType").and_then(Value::as_str).unwrap_or("");

            let removable = is_removable_windows(media_type, interface_type);
            if !include_system && !removable && self.is_system_disk(&device_path) {
                continue;
            }

            let mount_point = get_windows_mount_point(&device_path);

            devices.push(DeviceInfo {
                path: device_path,
                size,
                model,
                removable,
                mount_point,
            });
        }

        Ok(devices)
    }

    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()> {
        let index = disk_index(device_path).context("Could not parse disk index")?;
        let script = format!(
            "Get-Partition -DiskNumber {index} | Get-Volume | Where-Object {{ $_.DriveLetter }} | ForEach-Object {{ $_.DriveLetter }} | ForEach-Object {{ Dismount-Volume -DriveLetter $_ -Force -ErrorAction Stop }} | Out-String"
        );
        run_powershell(&script).map(|_| {
            tracing::info!(device = device_path, "Windows volumes unmounted");
        })?;
        Ok(())
    }

    fn is_system_disk(&self, _device_path: &str) -> bool {
        let Some(index) = disk_index(_device_path) else {
            return false;
        };

        let script = format!(
            "Get-Disk -Number {index} | Select-Object IsBoot,IsSystem | ConvertTo-Json -Compress"
        );
        let Ok(output) = run_powershell(&script) else {
            return false;
        };
        let Ok(entries) = parse_json_array(&output) else {
            return false;
        };
        if entries.is_empty() {
            return false;
        }
        let entry = &entries[0];
        let is_boot = entry
            .get("IsBoot")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let is_system = entry
            .get("IsSystem")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        is_boot || is_system
    }
}

fn disk_index(device_path: &str) -> Option<u32> {
    let normalized = device_path.to_ascii_lowercase();
    let marker = r"\\.\physicaldrive";
    let idx = normalized.strip_prefix(marker)?;
    idx.parse().ok()
}

fn is_removable_windows(media_type: &str, interface_type: &str) -> bool {
    let media = media_type.to_ascii_lowercase();
    let interface = interface_type.to_ascii_lowercase();
    media.contains("removable") || media.contains("external") || interface == "usb"
}

fn run_powershell(script: &str) -> Result<String> {
    let mut last_error = String::new();

    for shell in &POWERSHELL_CANDIDATES {
        let output = match Command::new(shell)
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                last_error = format!("Failed to execute {shell}: {err}");
                continue;
            }
        };

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return Ok(text.trim().trim_start_matches('\u{feff}').to_string());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        last_error = stderr.to_string();
    }

    Err(anyhow::anyhow!(last_error))
}

fn parse_json_array(output: &str) -> Result<Vec<Value>> {
    let output = output.trim().trim_start_matches('\u{feff}');
    if output.is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(output)
        .context("Failed to parse PowerShell JSON output")?;
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(_) => Ok(vec![value]),
        _ => Ok(Vec::new()),
    }
}

fn get_windows_mount_point(device_path: &str) -> Option<String> {
    let Some(index) = disk_index(device_path) else {
        return None;
    };

    let script = format!(
        "Get-Partition -DiskNumber {index} | Get-Volume | Where-Object {{ $_.DriveLetter }} | ForEach-Object {{ $_.DriveLetter }}"
    );

    let Ok(output) = run_powershell(&script) else {
        return None;
    };
    output
        .lines()
        .map(|s| s.trim().to_string())
        .find(|s| !s.is_empty())
        .map(|letter| format!("{}:\\", letter))
}
