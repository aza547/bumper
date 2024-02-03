use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{self, BufRead, Write};
use chrono::prelude::*;
use regex::Regex;

/// Automated changelog version bumping tool.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum Commands {
  /// Major version bump
  Major,
  /// Minor version bump
  Minor,
  /// Patch version bump
  Patch,
}

fn main() {
  let cli: Cli = Cli::parse();

  match cli.command {
    Some(c) => bump(c),
    None => println!("Invalid command, try --help."),
  }
}

fn bump(flavor: Commands) {
  let path = "CHANGELOG.md";

  let lines: Vec<String> = read_file_lines(&path)
    .expect("Error reading file");
  
  let mut last_line: &str = "";
  let mut last_version: &str = "";

  let mut changed_empty = true;
  let mut added_empty = true;
  let mut fixed_empty = true;

  let regex_pattern = Regex::new(r"## \[(\d+.\d+.\d+)\] -.*").unwrap();

  for line in lines.iter() {
    if last_line == "### Changed" && line.starts_with("-") {
      changed_empty = false;
    } 

    if last_line == "### Added" && line.starts_with("-") {
      added_empty = false;
    } 

    if last_line == "### Fixed" && line.starts_with("-") {
      fixed_empty = false;
    } 
    
    if let Some(captures) = regex_pattern.captures(line) {
      last_version = captures.get(1).unwrap().as_str();
    }

    if line.starts_with("## [") {
      break;
    }

    last_line = line;
  }

  let mut output_lines: Vec<String> = Vec::new();
  let mut passed_last_release: bool = false;

  for line in lines.iter() {
    if line == "## Unreleased" {
      // Add headings for next unreleased section.
      output_lines.push("## Unreleased".to_owned());
      output_lines.push("### Changed".to_owned());
      output_lines.push("### Added".to_owned());
      output_lines.push("### Fixed\n".to_owned());

      // Update the current unreleased section to reflect the new version and date.
      let date: String = get_formatted_date();
      let new_version = get_new_version(&flavor, last_version);
      let prefix: String = format!("## [{}] - ", new_version).to_owned();
      output_lines.push(prefix + &date);
    } else if !passed_last_release && line == "### Changed"{
      if !changed_empty {output_lines.push(line.to_string())};
    } else if !passed_last_release && line == "### Added" {
      if !added_empty {output_lines.push(line.to_string())};
    } else if !passed_last_release && line == "### Fixed" {
      if !fixed_empty {output_lines.push(line.to_string())};
    } else if line.starts_with("## [") {
      passed_last_release = true;
      output_lines.push(line.to_string());
    } else {
      output_lines.push(line.to_string());
    }
  }

  rewrite_changelog(&path.to_string(), &output_lines);
}

fn read_file_lines(file_path: &str) -> io::Result<Vec<String>> {
  let file = File::open(file_path)?;
  let reader = io::BufReader::new(file);
  reader.lines().collect()
}

fn get_formatted_date() -> String {
  let local: DateTime<Local> = Local::now();
  local.format("%Y-%m-%d").to_string()
}

fn get_new_version(flavor: &Commands, last: &str) -> String {
  let pattern = Regex::new(r"(\d+)\.(\d+)\.(\d+)").unwrap();

  let mut major: i32 = 0;
  let mut minor: i32 = 0;
  let mut patch: i32 = 0;

  if let Some(captures) = pattern.captures(last) {
    major = captures.get(1).unwrap().as_str().parse().unwrap();
    minor = captures.get(2).unwrap().as_str().parse().unwrap();
    patch = captures.get(3).unwrap().as_str().parse().unwrap();
  }

  match flavor {
    Commands::Major => major += 1,
    Commands::Minor => minor += 1,
    Commands::Patch => patch += 1,
  }

  format!("{}.{}.{}", major, minor, patch)
}

fn rewrite_changelog(path: &String, lines: &Vec<String>) {
  let mut file = File::create(path).unwrap();

  for line in lines {
    writeln!(file, "{}", line).unwrap();
  }
}