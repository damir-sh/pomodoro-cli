// Import necessary crates for command-line parsing, I/O operations, threading, and time handling
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

// Define the main CLI structure using clap's derive macros
// This struct represents the top-level command-line interface for our Pomodoro timer
#[derive(Parser)]
#[command(name = "pomodoro", version, about = "Tiny Pomodoro CLI")]
struct Cli {
    // The CLI has a single field that holds the subcommand the user wants to execute
    #[command(subcommand)]
    command: Command,
}

// Define the available subcommands for the CLI
// Currently we only have one subcommand "run", but this enum structure allows
// for easy addition of more commands in the future (like "status", "config", etc.)
#[derive(Subcommand)]
enum Command {
    /// Run a Pomodoro cycle
    Run {
        /// Focus minutes - how long each focus session should last
        /// Default is 25 minutes, which is the traditional Pomodoro technique duration
        #[arg(short = 'f', long, default_value_t = 25)]
        focus: u64,
        /// Break minutes - how long each break should last
        /// Default is 5 minutes for short breaks between focus sessions
        #[arg(short = 'b', long, default_value_t = 5)]
        break_min: u64,
        /// Number of focus sessions before a long break (we'll use later)
        /// Default is 4 cycles, following the traditional Pomodoro technique
        #[arg(short = 'c', long, default_value_t = 4)]
        cycles: u64,
        /// Long break minutes
        /// Default is 15 minutes, which is longer than regular breaks for better rest
        #[arg(long = "long-break", default_value_t = 15)]
        long_break: u64,
        /// Take a long break every N focus sessions
        /// Default is every 4 sessions, aligning with traditional Pomodoro cycles
        #[arg(long = "long-every", default_value_t = 4)]
        long_every: u64,
    },
}

// Helper function to format seconds into MM:SS format for display
// This makes the countdown timer more readable by showing time in familiar format
// Example: 125 seconds becomes "2:05"
fn fmt_mm_ss(total_secs: u64) -> String {
    let m: u64 = total_secs / 60; // Extract minutes by integer division
    let s: u64 = total_secs % 60; // Extract remaining seconds using modulo operator
    format!("{m}:{s:02}") // Format with zero-padded seconds (e.g., "5:03" not "5:3")
}

// Main countdown function that displays a real-time timer
// This function creates a visual countdown that updates every second
// It uses precise timing to avoid drift over long periods
fn countdown_secs(secs: u64, label: &str) {
    let start: Instant = Instant::now(); // Record the exact moment we started counting
    let mut tick: u64 = 0u64; // Track how many seconds have elapsed since start

    // Main countdown loop - runs once per second until time expires
    loop {
        // Calculate how many seconds remain at this tick
        // saturating_sub prevents underflow if tick somehow exceeds secs
        let remaining = secs.saturating_sub(tick);

        // Render the current countdown state
        // \r (carriage return) moves cursor to start of line, overwriting previous output
        // This creates the effect of a timer that updates in place rather than scrolling
        print!("\r{label}: {}", fmt_mm_ss(remaining));
        io::stdout().flush().ok(); // Force output to display immediately (stdout is buffered)

        // Check if countdown is complete
        if remaining == 0 {
            println!(); // Add newline after finishing countdown to move to next line
            break; // Exit the countdown loop
        }

        // Schedule next tick exactly 1 second from start + current tick count
        // This approach prevents cumulative timing drift that would occur with
        // simple sleep(1 second) calls, which can accumulate small errors
        tick += 1;
        let target: Instant = start + Duration::from_secs(tick);
        let now: Instant = Instant::now();

        // Sleep until the target time, or skip if we're running late
        // This handles cases where the system is under load or hibernates
        if target > now {
            thread::sleep(target - now); // Sleep for the remaining time until next tick
        } else {
            // We're late (system hiccup, sleep, etc.) — skip sleeping to catch up
            // The next iteration will recalculate and try to get back on schedule
        }
    }
}

// Main entry point of the application
// This function orchestrates the entire Pomodoro session based on user input
fn main() {
    // Parse command-line arguments using clap
    // This will automatically handle --help, --version, and argument validation
    let cli: Cli = Cli::parse();

    // Handle the parsed command using pattern matching
    // Currently only handles the Run command, but structure allows easy extension
    match cli.command {
        Command::Run {
            focus,
            break_min,
            cycles,
            long_break,
            long_every,
        } => {
            // Display the configuration for this pomodoro session
            // This helps users confirm they've set the right parameters
            println!("Run with focus={focus}m, break-min={break_min}m, cycles={cycles}");

            // Convert minutes to seconds for the countdown functions
            // All our timing functions work in seconds for precision
            let focus_secs = focus * 60;

            // Run the specified number of pomodoro cycles
            // Each cycle consists of a focus period followed by a break (except the last)
            for n in 1..=cycles {
                // Display current session progress to help user track their progress
                println!("\n=== Session {n}/{cycles} ===");

                // Focus period - the main work time
                // This is when the user should focus on their task without distractions
                countdown_secs(focus_secs, "Focus");
                println!("✅ Focus done"); // Celebrate completion of focus time

                // Break period (skip break after the last session)
                // No need for a break after the final session since work is complete
                if n < cycles {
                    // Determine if this should be a long break or short break
                    // Long breaks occur every 'long_every' sessions for better rest
                    let is_long = n % long_every == 0;

                    // Calculate break duration based on break type
                    let break_secs = if is_long {
                        long_break * 60 // Convert long break minutes to seconds
                    } else {
                        break_min * 60 // Convert short break minutes to seconds
                    };

                    // Set appropriate label for the break type
                    let label = if is_long { "Long break" } else { "Break" };

                    // Run the break countdown with appropriate duration and label
                    countdown_secs(break_secs, label);
                    println!("☕ {label} over"); // Signal that break time is finished
                }
            }

            // Celebrate completion of all sessions
            // This provides positive reinforcement for completing the full Pomodoro session
            println!("\n🎉 All sessions done. Nice work.");
        }
    }
}
