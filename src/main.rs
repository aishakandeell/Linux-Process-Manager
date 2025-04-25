use chrono::Local;
use sysinfo::{System, Process, Pid, Signal};
use dialoguer::{Input, Confirm};
use std::fs::OpenOptions;
use std::io::Write;

struct ProcessManager {
    system: System,
}

impl ProcessManager {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    // List top N processes by CPU usage
    fn list_top_processes(&self, limit: usize) {
        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        println!("Top {} processes by CPU usage:", limit);
        for process in processes.iter().take(limit) {
            println!(
                "[PID: {}] {} - {:.2}% CPU",
                process.pid(),
                process.name(),
                process.cpu_usage()
            );
        }
    }

    // Check for high CPU usage (> threshold)
    fn check_high_cpu_usage(&self, threshold: f32) {
        for process in self.system.processes().values() {
            if process.cpu_usage() > threshold {
                println!(
                    "⚠️ High CPU Usage Detected: [PID: {}] {} - {:.2}% CPU",
                    process.pid(),
                    process.name(),
                    process.cpu_usage()
                );
            }
        }
    }

    // Prompt user to kill a process
    fn prompt_kill_process(&self, pid: Pid) {
        let process = self.system.process(pid);
        if let Some(proc) = process {
            if Confirm::new()
                .with_prompt(format!("Do you want to kill process '{}' [PID: {}]?", proc.name(), pid))
                .interact()
                .unwrap()
            {
                if proc.kill_with(Signal::Kill).is_some() {
                    let name = proc.name();
                    self.log_killed_process(pid, name);
                    println!("✅ Killed process '{}' (PID: {})", name, pid);
                } else {
                    println!("❌ Failed to kill process with PID {}", pid);
                }
            }
        } else {
            println!("⚠️ No process found with PID {}", pid);
        }
    }

    // Log killed process to file
    fn log_killed_process(&self, pid: Pid, name: &str) {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S");
        let entry = format!("[{}] Killed Process: {} (PID: {})\n", timestamp, name, pid);

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("lpm_log.txt")
            .expect("Failed to open log file");

        file.write_all(entry.as_bytes()).expect("Failed to write to log file");
    }
}

fn main() {
    let process_manager = ProcessManager::new();

    loop {
        println!("1. List Top Processes");
        println!("2. Check High CPU Usage");
        println!("3. Kill a Process");
        println!("4. Exit");

        let choice: u32 = Input::new()
            .with_prompt("Please choose an option")
            .interact_text()
            .unwrap();

        match choice {
            1 => {
                let limit: usize = Input::new()
                    .with_prompt("Enter the number of top processes to list")
                    .default(10)
                    .interact_text()
                    .unwrap();
                process_manager.list_top_processes(limit);
            }
            2 => {
                let threshold: f32 = Input::new()
                    .with_prompt("Enter the CPU usage threshold for high usage")
                    .default(80.0)
                    .interact_text()
                    .unwrap();
                process_manager.check_high_cpu_usage(threshold);
            }
            3 => {
                let pid_input: String = Input::new()
                    .with_prompt("Enter the PID of the process to kill")
                    .interact_text()
                    .unwrap();

                if let Ok(pid) = pid_input.parse::<u32>() {
                    process_manager.prompt_kill_process(Pid::from(pid as usize));

                } else {
                    println!("❗ Invalid PID input!");
                }
            }
            4 => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid option, please try again."),
        }
    }
}
