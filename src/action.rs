use anyhow::Error;
use crate::project_management::Requirements;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{fs::{File, OpenOptions, self}, io::{Read, self, Write}, path::PathBuf, process::Command};
use strum_macros::Display;
use termion::{color, style};

const WORKING_DIRECTORY: &str = "./ai_working_directory"; // TODO: Make this part of configuration.

#[derive(Clone, Debug, Deserialize, Display, PartialEq, Serialize,)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    AddDependency {directory: String, cli_arguments: Vec<String>},
    AppendToFile {file: String, content: String},
    BuildProject {directory: String, cli_arguments: Vec<String>},
    DeleteFile {file: String},
    DeleteFolder {directory: String},

    /// bool is currently used to make the json required from the bot easier to parse
    ///    enums without associated data are expected to be formatted differently in serde.
    ///    essentially, I am saying this bool does nothing at present.
    ///    TODO: Mitigate this through custom deserializer, or actually create a summarize method.
    GatherRequirements {summarize: bool},
    InitializeProject {directory: String, cli_arguments: Vec<String>},
    PackageSearch {cli_arguments: Vec<String>},

    /// bool is currently used to make the json required from the bot easier to parse
    ///    enums without associated data are expected to be formatted differently in serde.
    ///    essentially, I am saying this bool does nothing at present.
    ///    TODO: Mitigate this through custom deserializer
    ProjectCompleted {close: bool},
    ReadFile {file: String},
    RemoveDependency {directory: String, cli_arguments: Vec<String>},
    RenameFile {current_file: String, new_file: String},
    RunProject {directory: String, cli_arguments: Vec<String>},
    SearchDirectory {directory: String},
    TestProject {directory: String, cli_arguments: Vec<String>},
    UpdateProjectDependencies {directory: String, cli_arguments: Vec<String>},
    WriteFile {file: String, content: String},
}

#[derive(Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    CommandOutput(String),
    DirectoryContents(Vec<String>),
    FileContents(String),
    FileCreated(String, u64), // u64 is file size in bytes after creation.
    FileDeleted(String),
    FileModified(String, u64, u64), // u64 is before/after of file size in bytes.
    FileRenamed(String, String),
    FolderDeleted(String),
    Requirements(Requirements),
    Success,
}

impl Action {
    pub fn print(&self) {
        print!("{}{}{}{}{}", style::Bold, color::Fg(color::Rgb(183,185,142)), self.to_string(), style::Reset, color::Fg(color::Reset));
        match self {
                Self::AddDependency                 {directory, cli_arguments}
            |   Self::BuildProject                  {directory, cli_arguments}
            |   Self::InitializeProject             {directory, cli_arguments}
            |   Self::RemoveDependency              {directory, cli_arguments}
            |   Self::RunProject                    {directory, cli_arguments}
            |   Self::TestProject                   {directory, cli_arguments}
            |   Self::UpdateProjectDependencies     {directory, cli_arguments} 
            => {
                print!(": directory({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), directory, style::Reset, color::Fg(color::Reset));
                print!(" & cli_arguments[");
                for arg in cli_arguments {
                    print!(" {}{}{}{}{} ", style::Bold, color::Fg(color::Rgb(183,185,142)), arg, style::Reset, color::Fg(color::Reset));
                }
                print!("]");
            }
                Self::AppendToFile  {file, content}
            |   Self::WriteFile     {file, content} 
            => {
                print!(": file({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), file, style::Reset, color::Fg(color::Reset));
                print!(": content({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), content, style::Reset, color::Fg(color::Reset));
            }
                Self::DeleteFile    {file}
            |   Self::ReadFile      {file}
            => {
                print!(": file({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), file, style::Reset, color::Fg(color::Reset));
            }
                Self::DeleteFolder      {directory}
            |   Self::SearchDirectory   {directory}
            => {
                print!(": directory({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), directory, style::Reset, color::Fg(color::Reset));
            }
                Self::GatherRequirements {..}
            |   Self::ProjectCompleted {..}
            => {()}
            Self::RenameFile {current_file, new_file} => {
                print!(": current_file({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), current_file, style::Reset, color::Fg(color::Reset));
                print!(": new_file({}{}{}{}{})", style::Bold, color::Fg(color::Rgb(183,185,142)), new_file, style::Reset, color::Fg(color::Reset));
            }
            Self::PackageSearch { cli_arguments } => {
                print!(": cli_arguments[");
                for arg in cli_arguments {
                    print!(" {}{}{}{}{} ", style::Bold, color::Fg(color::Rgb(183,185,142)), arg, style::Reset, color::Fg(color::Reset));
                }
                print!("]");
            }
        }

        println!();
    }

    /// Executes the action represented by the enum variant.
    ///
    /// # Returns
    ///
    /// A `Result<ActionResult, io::Error>` containing the result of the executed action.
    pub fn run(&self) -> Result<ActionResult, Error> {
        match self {

            // Cargo Commands
                Self::AddDependency             {directory, cli_arguments} 
            |   Self::BuildProject              {directory, cli_arguments}
            |   Self::InitializeProject         {directory, cli_arguments}
            |   Self::RemoveDependency          {directory, cli_arguments}
            |   Self::RunProject                {directory, cli_arguments}
            |   Self::TestProject               {directory, cli_arguments}
            |   Self::UpdateProjectDependencies {directory, cli_arguments}
            => {
                let command = match self {
                    Self::AddDependency {..}                => "add",
                    Self::BuildProject {..}                 => "build",
                    Self::InitializeProject {..}            => "init",
                    Self::RemoveDependency {..}             => "remove",
                    Self::RunProject {..}                   => "run",
                    Self::TestProject {..}                  => "test",
                    Self::UpdateProjectDependencies {..}    => "update",
                    _ => panic!("We shouldn't be here.")
                };

                cargo_command_action(command, directory, cli_arguments)
            }
            Self::PackageSearch {cli_arguments} => {
                cargo_command_action("search", WORKING_DIRECTORY, cli_arguments)
            }

            Self::AppendToFile{file, content} => {
                let file_path = get_working_path(file);
                let metadata_before = std::fs::metadata(file_path.clone()).ok();
                let mut file_ = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file_path.clone())?;
                file_.write_all(content.as_bytes())?;
                let file_size_after = file_.metadata()?.len();
                let file_size_before = metadata_before.map(|m| m.len()).unwrap_or(0);
                Ok(ActionResult::FileModified(
                    file.clone(),
                    file_size_before,
                    file_size_after,
                ))
            }
            Self::DeleteFile{file} => {
                let file_path = get_working_path(file);
                std::fs::remove_file(file_path.clone())?;
                Ok(ActionResult::FileDeleted(file.clone()))
            }
            Self::DeleteFolder{directory} => {
                let folder_path = get_working_path(directory);
                std::fs::remove_dir_all(&folder_path)?;
                Ok(ActionResult::FolderDeleted(directory.clone()))
            }
            Self::GatherRequirements{..} => {
                let file_path = get_working_path("requirements.json");
                let mut file = File::open(file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let requirements: Requirements = from_str(&contents)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                Ok(ActionResult::Requirements(requirements))
            }
            Self::ReadFile{file} => {
                let file_path = get_working_path(file);
                let mut file = File::open(file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                Ok(ActionResult::FileContents(contents))
            }
            Self::RenameFile {current_file, new_file} => {
                let current_file_path = get_working_path(current_file);
                let new_file_path = get_working_path(new_file);
                std::fs::rename(current_file_path.clone(), new_file_path.clone())?;
                Ok(ActionResult::FileRenamed(
                    current_file.clone(),
                    new_file.clone(),
                ))
            }
            Self::SearchDirectory{directory: directory_path} => {
                let dir_path = get_working_path(directory_path);
                let mut contents: Vec<String> = Vec::new();
                for entry in fs::read_dir(&dir_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    let relative_path = get_output_path(path);
                    contents.push(relative_path);
                }
                Ok(ActionResult::DirectoryContents(contents))
            }
            Self::WriteFile{file, content} => {
                let file_path = get_working_path(file);
                let mut file_ = File::create(file_path.clone())?;
                file_.write_all(content.as_bytes())?;
                let file_size = file_.metadata()?.len();
                Ok(ActionResult::FileCreated(file.clone(), file_size))
            }
            _ => Ok(ActionResult::Success)
        }
    }
}

impl ActionResult {
    pub fn print(&self) {
        print!("{}{}{}{}{}", style::Bold, color::Fg(color::Rgb(183,185,142)), self.to_string(), style::Reset, color::Fg(color::Reset));
        match self {
                Self::CommandOutput(to_print) 
            |   Self::FileContents(to_print)
            |   Self::FileDeleted(to_print)
            |   Self::FolderDeleted(to_print)
            => { print!(": {}{}{}", color::Fg(color::Rgb(183,185,142)), to_print, color::Fg(color::Reset));}
            Self::DirectoryContents(contents) => {
                print!(": contents[");
                for content in contents {
                    print!(" {}{}{} ", color::Fg(color::Rgb(183,185,142)), content, color::Fg(color::Reset));
                }
                print!("]");
            }
            Self::FileCreated(file, size_in_bytes) => {
                print!(": file[{}{}{}]", color::Fg(color::Rgb(183,185,142)), file, color::Fg(color::Reset));
                print!(" & size_in_bytes[{}{}{}]", color::Fg(color::Rgb(183,185,142)), size_in_bytes, color::Fg(color::Reset));
            }
            Self::FileModified(file, size_in_bytes_before, size_in_bytes_after) => {
                print!(": file[{}{}{}]", color::Fg(color::Rgb(183,185,142)), file, color::Fg(color::Reset));
                print!(" & size_in_bytes_before[{}{}{}]", color::Fg(color::Rgb(183,185,142)), size_in_bytes_before, color::Fg(color::Reset));
                print!(" & size_in_byte_after[{}{}{}]", color::Fg(color::Rgb(183,185,142)), size_in_bytes_after, color::Fg(color::Reset));
            } // u64 is before/after of file size in bytes.
            Self::FileRenamed(file_before, file_after) => { 
                print!(": file_name_before[{}{}{}]", color::Fg(color::Rgb(183,185,142)), file_before, color::Fg(color::Reset));
                print!(" & file_name_after[{}{}{}]", color::Fg(color::Rgb(183,185,142)), file_after, color::Fg(color::Reset));
            }
            Self::Requirements(_) => {
                print!(": placeholder text b/c requirements are long.");
            }
            _ => {()}
        }
        println!();
        println!();
    }
}

fn cargo_command_action(cargo_command: &str, project_directory_path: &str, cli_arguments: &Vec<String>) -> Result<ActionResult, Error> {
    let project_directory_path = get_working_path(project_directory_path);
    let output = Command::new("cargo")
        .arg(cargo_command)
        .args(cli_arguments)
        .current_dir(project_directory_path)
        .output()?;
    let output_string = match output.status.success() {
        true => String::from_utf8(output.stdout)?,
        false => String::from_utf8(output.stderr)?
    };
    Ok(ActionResult::CommandOutput(output_string))
}

fn get_working_path(path: &str) -> PathBuf {
    PathBuf::from(WORKING_DIRECTORY).join(path)
}

fn get_output_path(path: PathBuf) -> String {
    path.strip_prefix(WORKING_DIRECTORY).unwrap().to_string_lossy().into_owned()
}