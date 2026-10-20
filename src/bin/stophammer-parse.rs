//! CLI binary: reads RSS XML from stdin, writes `IngestFeedData` JSON to stdout.
//!
//! Exit codes:
//! - 0: success
//! - 1: parse error (JSON error object written to stderr)
//! - 2: no input / I/O error

use std::io::Read;
use std::process::ExitCode;

use stophammer_parser::phase::Phase;
use stophammer_parser::profile;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let mut phase_filter: Option<Vec<Phase>> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--phases" {
            i += 1;
            if i < args.len() {
                phase_filter = Some(parse_phases(&args[i]));
            }
        }
        i += 1;
    }

    let mut xml = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut xml) {
        eprintln!("{{\"error\": \"failed to read stdin: {e}\"}}");
        return ExitCode::from(2);
    }

    if xml.trim().is_empty() {
        eprintln!("{{\"error\": \"no input\"}}");
        return ExitCode::from(2);
    }

    let parser = match phase_filter {
        Some(phases) => profile::stophammer_phases_only(&phases),
        None => profile::stophammer(),
    };

    match parser.parse(&xml) {
        Ok(feed) => match serde_json::to_string(&feed) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{{\"error\": \"JSON serialization failed: {e}\"}}");
                ExitCode::from(1)
            }
        },
        Err(e) => {
            eprintln!("{{\"error\": \"{e}\"}}");
            ExitCode::from(1)
        }
    }
}

fn parse_phases(s: &str) -> Vec<Phase> {
    s.split(',')
        .filter_map(|p| match p.trim().to_lowercase().as_str() {
            "rss2core" | "rss2" => Some(Phase::Rss2Core),
            "itunes" => Some(Phase::Itunes),
            "phase1" | "1" => Some(Phase::Phase1),
            "phase2" | "2" => Some(Phase::Phase2),
            "phase3" | "3" => Some(Phase::Phase3),
            "phase4" | "4" => Some(Phase::Phase4),
            "phase5" | "5" => Some(Phase::Phase5),
            "phase6" | "6" => Some(Phase::Phase6),
            "pending" => Some(Phase::Pending),
            _ => None,
        })
        .collect()
}
