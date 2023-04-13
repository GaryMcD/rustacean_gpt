use anyhow::{anyhow, Error};
use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionResponseMessage, CreateChatCompletionRequestArgs, Role}, Client, error::OpenAIError};
use crate::{Action, AiResponse, Configuration, InteractionType}; //, OpenAIChatCompletionMessage};
use serde_json::{from_str, to_string_pretty, Value};
use std::{fs::{File, OpenOptions, rename}, io::{Read, self, Write}, path::Path};
use termion::color;

pub(crate) struct MainLoop {
    configuration: crate::Configuration,

    conversation_history_file_path: String,
    pub current_action: MainLoopAction,
    number_of_messages_to_remove: i32,
}

impl MainLoop {
    pub(crate) fn new(configuration: Configuration) -> Result<Self, Error> {
        let conversation_history_file_path = String::from("");
        let current_action = MainLoopAction::Initialize;
        let number_of_messages_to_remove = 0;

        Ok(Self { 
                configuration,
                conversation_history_file_path, 
                current_action,
                number_of_messages_to_remove,
        })
    }

    pub(crate) async fn take_action(&mut self) -> Result<(), Error> {
        match &self.current_action {
            MainLoopAction::GetAiResponse => {
                let mut conversation_history = self.get_conversation_history()?;

                if self.number_of_messages_to_remove > 0 {
                    let number_of_messages_to_remove;
                    let max_number_of_messages_to_remove = (conversation_history.len() as i32) - 2;
                    if self.number_of_messages_to_remove > max_number_of_messages_to_remove {
                        number_of_messages_to_remove = max_number_of_messages_to_remove;
                    } else {
                        number_of_messages_to_remove = self.number_of_messages_to_remove;
                    }
                    for _ in 0..number_of_messages_to_remove {
                        conversation_history.remove(1);
                    }
                }

                let ai_response = {                    
                    let client = Client::new();

                    let request = CreateChatCompletionRequestArgs::default()
                        .model("gpt-3.5-turbo")
                        //.model("gpt-4-32k")
                        .messages(conversation_history)
                        .build()?;

                    match client.chat().create(request).await {
                        Ok(chat) => chat,
                        Err(error) => {
                            match error {
                                OpenAIError::ApiError(error) => {
                                    if error.message.contains("This model's maximum context length") {
                                        println!("{}{}{}", color::Fg(color::Red), format!("{:?}", error), color::Fg(color::Reset));
                                        self.number_of_messages_to_remove = self.number_of_messages_to_remove + 5;
                                        return Ok(())
                                    } else {
                                        println!("{}{}{}", color::Fg(color::Red), format!("{:?}", error), color::Fg(color::Reset));
                                        self.current_action = MainLoopAction::Exit;
                                        return Err(anyhow!("Error getting AI Response. This specific error is not handled."))
                                    }
                                }, 
                                _ => { 
                                    println!("{}{}{}", color::Fg(color::Red), format!("{:?}", error), color::Fg(color::Reset));
                                    self.current_action = MainLoopAction::Exit;
                                    return Err(anyhow!("Error getting AI Response. This specific error is not handled."))
                                }
                            }
                        }
                    }
                };

                let new_message = ai_response.choices[0].clone().message.content;
                let role = Role::Assistant;

                pretty_print_ai_response(&new_message);

                self.current_action = MainLoopAction::SaveMessage(new_message, role);
            }
            MainLoopAction::GetUserInput => {
                let user_input = {
                    let user_input_result = get_user_input();

                    user_input_result?
                };

                if user_input.to_lowercase() == "exit" {
                    self.current_action = MainLoopAction::Exit;
                } else {
                    self.current_action = MainLoopAction::SaveMessage(user_input, Role::User);
                }
            }
            MainLoopAction::Initialize => {
                self.conversation_history_file_path = self.initialize_conversation_history_file()?;
                self.current_action = MainLoopAction::SaveMessage(self.configuration.initial_prompt.clone(), Role::System);
            }
            MainLoopAction::SaveMessage(system_message, role) => {
                self.save_message_to_conversation_history(system_message.clone(), role.clone())?;
                self.current_action = match (self.configuration.interaction_type.clone(), role) {
                    (_, Role::System) | (_, Role::User) => MainLoopAction::GetAiResponse,
                    (InteractionType::Interactive, Role::Assistant) => MainLoopAction::GetUserInput,
                    (InteractionType::Autonomous, Role::Assistant) => MainLoopAction::TakeAutonomousAction,
                };
            }
            MainLoopAction::TakeAutonomousAction => {
                let conversation_history = self.get_conversation_history()?;
                let last_message = conversation_history.last();
                let ai_response: Result<AiResponse, serde_json::Error> = match last_message {
                    Some(message) => {
                        if message.role != Role::Assistant {
                            return Err(anyhow!("Last message was not from assistant. Thus it is not an autonomous action to take. We have arrived here by error."));
                        } else {
                             serde_json::from_str(&message.content)
                        }
                    },
                    None => return Err(anyhow!("Error getting last message from conversation history"))
                };


                let action_result = match ai_response {
                    Ok(ai_response) => {
                        match ai_response.next_command {
                            Action::ProjectCompleted{..} => {
                                self.current_action = MainLoopAction::Exit;
                                return Ok(())
                            },
                            _ => {
                                let next_command_result = ai_response.next_command.run();
                                match next_command_result {
                                    Ok(result) => {result.print(); format!("{:?}", result)}
                                    Err(error) => {error_printing(&format!("{:?}", error)); format!("{:?}", error)}
                                }
                            }
                        }
                    },
                    Err(error) => {
                        let response_message = format!("{:?}\n{}", error, self.configuration.help_text_for_ai_parsing_errors);
                        error_printing(&format!("{:?}", error));
                        response_message
                    }
                };

                self.current_action = MainLoopAction::SaveMessage(format!("{:?}", action_result), Role::System);

            }
            _ => ()
        }

        Ok(())
    }

    fn append_message_to_conversation_history_file(&self, new_message: ChatCompletionResponseMessage) -> io::Result<()> {
        // Read the file contents
        let mut file = File::open(self.conversation_history_file_path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize the existing JSON array
        let mut json_array: Vec<Value> = from_str(&contents).unwrap_or_else(|_| Vec::new());

        // Serialize the object and append it to the JSON array
        let json_value = serde_json::to_value(&new_message).unwrap();
        json_array.push(json_value);

        // Serialize the updated JSON array
        let updated_contents = to_string_pretty(&json_array)?;

        // Write the updated JSON array to the file
        let mut file = OpenOptions::new().write(true).truncate(true).open(self.conversation_history_file_path.clone())?;
        file.write_all(updated_contents.as_bytes())?;

        Ok(())
    }

    fn get_conversation_history(&self) -> Result<Vec<ChatCompletionRequestMessage>, Error> {
        // Read the file contents
        let mut file = File::open(self.conversation_history_file_path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let messages: Vec<ChatCompletionRequestMessage> = from_str(&contents)?;

        Ok(messages)
    }

    fn get_next_available_file_name(&self) -> String {
        let base_path = &self.configuration.conversation_history_directory;
        let file_name = match self.configuration.interaction_type {
            InteractionType::Autonomous => &self.configuration.base_autonomous_conversation_history_file_name,
            InteractionType::Interactive => &self.configuration.base_interactive_conversation_history_file_name,
        };
    
    
        let mut counter = 1;
    
        loop {
            let new_file_name = format!("{}{:05}.json", file_name, counter);
            let new_file_path = format!("{}/{}", base_path, new_file_name);
    
            if !Path::new(&new_file_path).exists() {
                return new_file_path;
            }
    
            counter += 1;
        }
    }

    fn initialize_conversation_history_file(&self) -> io::Result<String> {
        match self.configuration.interaction_type {
            InteractionType::Autonomous => {
                let conversation_history_path = 
                    format!(
                        "{}/{}.json", 
                        self.configuration.conversation_history_directory, 
                        self.configuration.base_autonomous_conversation_history_file_name);
                
                if Path::new(&conversation_history_path).exists() {
                    let new_file_name = self.get_next_available_file_name();
                    rename(&conversation_history_path, &new_file_name)?;
                }
    
                File::create(conversation_history_path.clone())?;
    
                Ok(conversation_history_path)
            },
            InteractionType::Interactive => Ok("".to_string())
        }
    }

    fn save_message_to_conversation_history(&self, message: String, role: Role) -> io::Result<()> {
        let new_response_message = ChatCompletionResponseMessage { role, content: message.to_string() };
        self.append_message_to_conversation_history_file(new_response_message)
    }
}

fn get_user_input() -> Result<String, std::io::Error> {
    print!("User Response: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Remove trailing newline
    Ok(input.trim().to_string())
}

pub(crate) enum MainLoopAction {
    Exit,
    GetAiResponse,
    GetUserInput,
    Initialize,
    SaveMessage(String, Role),
    TakeAutonomousAction,
}

fn error_printing(unparseable: &str) {
    println!();
    println!("{}{}{}", color::Fg(color::Red), "Error Printing Natively:", color::Fg(color::Reset));
    println!("   {:?}", unparseable);
    println!();
}

fn pretty_print_ai_response(ai_response: &str) {
    match serde_json::from_str::<AiResponse>(ai_response) {
        Ok(ai_response) => ai_response.print(),
        Err(_) => error_printing(ai_response),
    }
}