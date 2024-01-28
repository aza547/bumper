use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use chrono::prelude::*;
use std::process;
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
  let cli = Cli::parse();

  match cli.command {
    Some(c) => bump(c),
    None => println!("Invalid command, try --help."),
  }
}

fn bump(flavor: Commands) {
  println!("{:?}", flavor);

  // TODO search recursively for this
  let path = "docs/CHANGELOG.md";

  if let Ok(metadata) = fs::metadata(path) {
    if !metadata.is_file() {
      println!("Not a file.");
      return;
    }
  } else {
    println!("No CHANGELOG.md found.");
    return;
  }

  let lines: Result<Vec<String>, io::Error> = read_file_lines(path);

  let lines: Vec<String> = lines.unwrap_or_else(|err| {
    eprintln!("Error reading the file: {}", err);
    process::exit(1);
  });

  let mut last_version: Option<String> = None;
  
  let mut last: &str = "";
  let mut changed_empty = true;
  let mut added_empty = true;
  let mut fixed_empty = true;

  let regex_pattern = Regex::new(r"## \[(\d+.\d+.\d+)\] -.*").unwrap();

  for line in lines.iter() {
    if last == "### Changed" && line.starts_with("-") {
      changed_empty = false;
    } 

    if last == "### Added" && line.starts_with("-") {
      added_empty = false;
    } 

    if last == "### Fixed" && line.starts_with("-") {
      fixed_empty = false;
    } 
    
    if let Some(captures) = regex_pattern.captures(line) {
      last_version = Some(captures.get(1).unwrap().as_str().to_owned());
    }

    last = line;

    if line.starts_with("## [") {
      break;
    }
  }

  let last_version: String = last_version.expect("latest_version is None");
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
      let new_version = get_new_version(&flavor, &last_version);
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

  for (_, line) in output_lines.iter().enumerate().take(20) {
    println!("{}", line);
  }

  rewrite_changelog(&path.to_string(), &output_lines);
}

fn read_file_lines(file_path: &str) -> io::Result<Vec<String>> {
  let file = File::open(file_path)?;
  let reader = io::BufReader::new(file);
  let lines: io::Result<Vec<String>> = reader.lines().collect();
  return lines;
}

fn get_formatted_date() -> String {
  let local: DateTime<Local> = Local::now();
  let formatted_date: String = local.format("%Y-%m-%d").to_string();
  return formatted_date;
}

fn get_new_version(flavor: &Commands, last: &String) -> String {
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

  return format!("{}.{}.{}", major, minor, patch);
}

fn rewrite_changelog(path: &String, lines: &Vec<String>) {
  let mut file = File::create(path).unwrap();

  for line in lines {
    writeln!(file, "{}", line).unwrap();
  }
}