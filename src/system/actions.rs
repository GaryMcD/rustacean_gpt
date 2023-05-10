// src/action.rs

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::{fs::{File, OpenOptions, self}, io::{Read, self, Write}, path::Path, process::Command};
use termion::color;

// Responsible for running local commands on the host machine
// Provides a safe and controlled way of executing commands

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize,)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    CargoRun { directory: String, arguments: String },
    CommandLine { command: String, arguments: Vec<String> },
    DeleteDirectory { directory: String },
    DeleteFile { file: String },
    ReadFile { file: String },
    SaveMemory { memory: String },
    SearchDirectory { directory: String },
    Standby { completed: bool },
    WriteFile { file: String, contents: String}
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    CommandOutput(String),
    DirectoryContents(Vec<String>),
    Failure(String),
    FileContents(String),
    Success,
}

impl Action {
    pub fn print(&self) {
        let val = self.to_variant_string();
        print!("{}{}{}", color::Fg(color::Rgb(183,185,142)), val, color::Fg(color::Reset));
    }

    pub fn take_action(&self, working_directory: String) -> Result<ActionResult, Error> {
        match self {
            Action::CargoRun { directory , arguments} => {
                let full_directory = Path::new(&working_directory).join(directory);

                let output = Command::new("cargo")
                    .arg("run")
                    .arg(arguments)
                    .current_dir(full_directory)
                    .output()
                    .map_err(|e| Error::new(e))?;

                let output_str = match output.status.success() {
                    true => String::from_utf8_lossy(&output.stdout).to_string(),
                    false => String::from_utf8_lossy(&output.stderr).to_string(),
                };

                Ok(ActionResult::CommandOutput(output_str))
            }

            Action::CommandLine { command, arguments } => {
                let output = Command::new(command)
                    .args(arguments)
                    .current_dir(working_directory)
                    .output()
                    .map_err(|e| Error::new(e))?;

                let output_str = format!("STDOUT: {} && STDERR: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));

                Ok(ActionResult::CommandOutput(output_str))
            }

            Action::DeleteDirectory { directory } => {
                let full_directory = Path::new(&working_directory).join(directory);
                fs::remove_dir_all(full_directory).map_err(|e| Error::new(e))?;
                Ok(ActionResult::Success)
            }

            Action::DeleteFile { file } => {
                let full_file = Path::new(&working_directory).join(file);
                fs::remove_file(full_file).map_err(|e| Error::new(e))?;
                Ok(ActionResult::Success)
            }

            Action::ReadFile { file } => {
                let full_file = Path::new(&working_directory).join(file);
                let mut file = File::open(full_file).map_err(|e| Error::new(e))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents).map_err(|e| Error::new(e))?;
                Ok(ActionResult::FileContents(contents))
            }

            Action::SearchDirectory { directory } => {
                let full_directory = Path::new(&working_directory).join(directory);

                let entries = fs::read_dir(full_directory)
                    .map_err(|e| Error::new(e))?
                    .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
                    .collect::<Result<Vec<String>, io::Error>>()
                    .map_err(|e| Error::new(e))?;

                Ok(ActionResult::DirectoryContents(entries))
            }

            Action::SaveMemory { .. } => {
                Ok(ActionResult::Success)
            }

            Action::Standby { .. } => {
                Ok(ActionResult::Success)
            }

            Action::WriteFile { file, contents } => {
                let full_file = Path::new(&working_directory).join(file);

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(full_file)
                    .map_err(|e| Error::new(e))?;

                file.write_all(contents.as_bytes()).map_err(|e| Error::new(e))?;
                Ok(ActionResult::Success)
            }
        }
    }

    pub fn to_variant_string(&self) -> String {
        match self {
            Action::CargoRun { directory, arguments } => {
                format!("Cargo Run: directory(\"{}\"), arguments(\"{}\")", directory, arguments)
            }
            Action::CommandLine { command, arguments } => {
                format!("Command Line: command(\"{}\"), arguments(\"{}\")", command, arguments.join(", "))
            }
            Action::DeleteDirectory { directory } => {
                format!("Delete Directory: directory(\"{}\")", directory)
            }
            Action::DeleteFile { file } => {
                format!("Delete File: file(\"{}\")", file)
            }
            Action::ReadFile { file } => {
                format!("Read File: file(\"{}\")", file)
            }
            Action::SaveMemory { memory } => {
                format!("Save Memory: memory(\"{}\")", memory)
            }
            Action::SearchDirectory { directory } => {
                format!("Search Directory: directory(\"{}\")", directory)
            }
            Action::Standby { completed } => {
                format!("Standby: completed(\"{}\")", completed)
            }
            Action::WriteFile { file, contents } => {
                format!("Write File: file(\"{}\"), contents(\"{}\")", file, contents)
            }
        }
    }
}

impl ActionResult {
    pub fn print(&self) {
        let val = self.to_variant_string();
        println!("{}{}{}", color::Fg(color::Rgb(143, 204, 191)), val, color::Fg(color::Reset));
    }

    pub fn to_variant_string(&self) -> String {
        match self {
            ActionResult::CommandOutput(output) => {
                format!("Command Output: {}", output)
            }
            ActionResult::DirectoryContents(contents) => {
                let contents_str = contents.join(", ");
                format!("Directory Contents: [{}]", contents_str)
            }
            ActionResult::Failure(failure_message) => {
                format!("Failure: {}", failure_message)
            }
            ActionResult::FileContents(contents) => {
                format!("File Contents: {}", contents)
            }
            ActionResult::Success => {
                format!("Success")
            }
        }
    }
}