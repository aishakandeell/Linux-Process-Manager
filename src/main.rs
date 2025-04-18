use sysinfo::{Process, System, Pid};
use dialoguer::{Confirm, Input};
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

//List top N processes by CPU usage
fn list_top_processes(limit: usize) {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

    println!("Top {} processes by CPU usage:", limit);
    for process in processes.iter().take(limit) {
        println!(
            "[PID: {}] {:?} - {:.2}% CPU",
            process.pid(),
            process.name(), // Uses {:?} to display OsStr
            process.cpu_usage()
        );
    }
}

// Warn about high-CPU processes
fn check_high_cpu_usage(threshold: f32) {
    let mut sys = System::new_all();
    sys.refresh_all();

    for process in sys.processes().values() {
        if process.cpu_usage() > threshold {
            println!(
                "⚠️ High CPU Usage: [PID: {}] {:?} - {:.2}% CPU",
                process.pid(),
                process.name(),
                process.cpu_usage()
            );
        }
    }
}

//Ask user if they want to kill a process
fn prompt_kill_process(pid: Pid, name: &OsStr) {
    if Confirm::new()
        .with_prompt(format!(
            "Do you want to kill process '{}' [PID: {}]?",
            name.to_string_lossy(),
            pid
        ))
        .interact()
        .unwrap()
    {
        let mut sys = System::new_all();
        sys.refresh_all();
        if let Some(process) = sys.process(pid) {
            if process.kill() {
                println!("Killed [PID: {}]", pid);
                log_killed_process(pid, &name.to_string_lossy());
            } else {
                println!("Failed to kill [PID: {}]", pid);
            }
        }
    }
}
fn log_killed_process(pid: Pid, name: &str) {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] Killed Process: {} (PID: {})\n", timestamp, name, pid);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("lpm_log.txt")
        .expect("Failed to open log file.");

    file.write_all(log_entry.as_bytes())
        .expect("Failed to write to log file.");
}

fn main() {
    let top_n = 10;
    list_top_processes(top_n);
    check_high_cpu_usage(80.0);

    let pid_input: usize = Input::new()
        .with_prompt("Enter PID to kill (or 0 to skip)")
        .interact_text()
        .unwrap();

    if pid_input != 0 {
        let pid = Pid::from(pid_input); // Converts usize to Pid
        let mut sys = System::new_all();
        sys.refresh_all();

        if let Some(proc) = sys.process(pid) {
            prompt_kill_process(proc.pid(), proc.name());
        } else {
            println!("⚠️ No process with PID {} found.", pid_input);
        }
    }
}
