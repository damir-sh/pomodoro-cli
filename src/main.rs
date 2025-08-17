// Import necessary crates for command-line parsing, I/O operations, threading, and time handling
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

// Define the main CLI structure using clap's derive macros
#[derive(Parser)]
#[command(name = "pomodoro", version, about = "Tiny Pomodoro CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

// Define the available subcommands for the CLI
#[derive(Subcommand)]
enum Command {
    /// Run a Pomodoro cycle
    Run {
        /// Focus minutes - how long each focus session should last
        #[arg(short = 'f', long, default_value_t = 25)]
        focus: u64,
        /// Break minutes - how long each break should last
        #[arg(short = 'b', long, default_value_t = 5)]
        break_min: u64,
        /// Number of focus sessions before a long break (we'll use later)
        #[arg(short = 'c', long, default_value_t = 4)]
        cycles: u64,
    },
}

// Helper function to format seconds into MM:SS format for display
fn fmt_mm_ss(total_secs: u64) -> String {
    let m: u64 = total_secs / 60; // Extract minutes
    let s: u64 = total_secs % 60; // Extract remaining seconds
    format!("{m}:{s:02}") // Format with zero-padded seconds
}

// Main countdown function that displays a real-time timer
fn countdown_secs(secs: u64, label: &str) {
    let start: Instant = Instant::now(); // Record when we started
    let mut tick: u64 = 0u64; // Track how many seconds have passed

    loop {
        // Calculate how many seconds remain at this tick
        let remaining = secs.saturating_sub(tick);

        // Render the current countdown state (carriage return overwrites the line)
        print!("\r{label}: {}", fmt_mm_ss(remaining));
        io::stdout().flush().ok(); // Force output to display immediately

        // Check if countdown is complete
        if remaining == 0 {
            println!(); // Add newline after finishing countdown
            break;
        }

        // Schedule next tick exactly 1 second from start + current tick count
        tick += 1;
        let target: Instant = start + Duration::from_secs(tick);
        let now: Instant = Instant::now();

        // Sleep until the target time, or skip if we're running late
        if target > now {
            thread::sleep(target - now);
        } else {
            // We're late (system hiccup) — skip sleeping to catch up
        }
    }
}

// Main entry point of the application
fn main() {
    // Parse command-line arguments
    let cli: Cli = Cli::parse();

    // Handle the parsed command
    match cli.command {
        Command::Run {
            focus,
            break_min,
            cycles,
        } => {
            // Display the configuration for this pomodoro session
            println!("Run with focus={focus}m, break-min={break_min}m, cycles={cycles}");

            // Convert minutes to seconds for the countdown functions
            let focus_secs = focus * 60;
            let break_secs = break_min * 60;

            // Run the specified number of pomodoro cycles
            for n in 1..=cycles {
                println!("\n=== Session {n}/{cycles} ===");

                // Focus period
                countdown_secs(focus_secs, "Focus");
                println!("✅ Focus done");

                // Break period (skip break after the last session)
                if n < cycles {
                    countdown_secs(break_secs, "Break");
                    println!("☕ Break over");
                }
            }

            // Celebrate completion of all sessions
            println!("\n🎉 All sessions done. Nice work.");
        }
    }
}
