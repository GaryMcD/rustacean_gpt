mod action;
mod ai_response;
mod main_loop;
mod project_management;

pub use action::Action;
pub use ai_response::AiResponse;

use anyhow::Error;
use crate::project_management::{Constraint, Details, Requirements, Risk, RiskDetails, SuccessCriteria};
use main_loop::{MainLoop, MainLoopAction};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{fs::{OpenOptions, read_dir, read_to_string, remove_dir, remove_file}, io::{stdout, Write}};
use termion::{clear, cursor};

#[tokio::main]
async fn main() -> Result<(), Error> {

    let mut stdout = stdout();
    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    let requirements = Requirements {
        stakeholder_needs: vec![
            "Print the Fibonnaci Sequence to an N depth.".to_string(),
            "N should be provided as a cli argument when calling your project".to_string(),
        ],
        functional_requirements: vec![],
        non_functional_requirements: vec![],
        constraints: vec![
            Constraint::Technology(Details {
                name: "Rust programming language".to_string(),
                details: "The application must be written in the Rust programming language.".to_string(),
                lower_bound: None,
                upper_bound: None,
            }),
        ],
        risks: vec![],
        success_criteria: vec![
            SuccessCriteria::QualityAssurance(Details {
                name: "Functionality correctness".to_string(),
                details: "The application should correctly print the Fibonacci Sequence.".to_string(),
                lower_bound: None,
                upper_bound: None,
            }),
            SuccessCriteria::QualityAssurance(Details {
                name: "Functionality correctness".to_string(),
                details: "The application should print to the correct depth.".to_string(),
                lower_bound: None,
                upper_bound: None,
            }),
        ],
    };

    delete_dir_contents("./ai_working_directory")?;

    let mut requirements_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open("./ai_working_directory/requirements.json")?;

    let requirements = serde_json::to_string(&requirements)?;

    requirements_file.write_all(requirements.as_bytes())?;

    let configuration_file_contents = read_to_string(".config/configuration.json")?;
    let mut configuration: Configuration = from_str(&configuration_file_contents)?;
    
    configuration.initial_prompt = read_to_string(".config/InitialSystemPrompt.txt")?;
    configuration.help_text_for_ai_parsing_errors = read_to_string(".config/HelpText.txt")?;

    let mut app = MainLoop::new(configuration)?;

    'main: loop {
        app.take_action().await?;
        match app.current_action {
            MainLoopAction::Exit => break 'main,
            _ => ()
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Configuration {
    base_autonomous_conversation_history_file_name: String,
    base_interactive_conversation_history_file_name: String,
    conversation_history_directory: String,
    interaction_type: InteractionType,
    model: String,

    #[serde(skip)]
    initial_prompt: String,

    #[serde(skip)]
    help_text_for_ai_parsing_errors: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) enum InteractionType {
    Autonomous,
    Interactive,
}

fn delete_dir_contents(path: &str) -> std::io::Result<()> {
    let path = std::path::PathBuf::from(path);

    for entry in read_dir(&path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            remove_file(&entry_path)?;
        } else {
            delete_dir_contents(entry_path.to_str().unwrap())?;
            remove_dir(&entry_path)?;
        }
    }

    Ok(())
}