use anyhow::Error;
use crate::{agent::{Agent, Response}, memory::{Memory, MemoryData}, configuration::ApplicationConfiguration};
use super::{Action, add_chat_to_conversation, Chat, Conversation, Whom, ActionResult};
use termion::{color, style};

enum LoopState {
    Exit,
    GetAIResponse,
    Initializing,
    TakeAction,
}

pub async fn application_loop(application_configuration: &ApplicationConfiguration, agent: Box<dyn Agent>, memory: &mut Box<dyn Memory>) -> Result<(), Error> {
    let mut loop_state = LoopState::Initializing;
    let mut unparsed_ai_response: Chat = Chat { text: "".to_string(), whom: Whom::System };
    let mut related_memories = vec![];

    'app: loop {
        let conversation = super::conversation(&application_configuration.system.conversation_file_path).await?.unwrap();
        loop_state = match loop_state {
            LoopState::Exit => break 'app,
            LoopState::GetAIResponse => get_ai_response(&agent, &conversation, &related_memories, &mut unparsed_ai_response).await?,
            LoopState::Initializing => initialize_loop(&agent, application_configuration, &conversation, &memory, &mut related_memories, &mut unparsed_ai_response).await?,
            LoopState::TakeAction => take_action(&agent, application_configuration, memory, &mut related_memories, &unparsed_ai_response).await?
        }
    }
    Ok(())
}

async fn take_action(agent: &Box<dyn Agent>, application_configuration: &ApplicationConfiguration, memory: &mut Box<dyn Memory>, related_memories: &mut Vec<MemoryData>, unparsed_ai_response: &Chat) -> Result<LoopState, Error> {
    let conversation_file_path = &application_configuration.system.conversation_file_path;
    let working_directory = &application_configuration.system.working_directory;

    add_chat_to_conversation(&conversation_file_path, unparsed_ai_response.clone()).await?;
    match unparsed_ai_response.parse() {
        Ok(response) => {
            response.print();
            match response.clone().next_command {
                Action::SaveMemory { memory: memory_as_string } => {
                    let memory_embedding = agent.get_string_embedding(&memory_as_string).await?;
                    _ = memory.add_memory(MemoryData(memory_embedding, memory_as_string)).await?;
                    
                    let action_result = ActionResult::Success;
                    process_successful_action_result(&action_result, agent, &conversation_file_path, &memory, related_memories, response).await
                },
                Action::Standby { .. } => Ok(LoopState::Exit),
                _ => {
                    let action_result = response.next_command.take_action(working_directory.clone());
                    match action_result {
                        Ok(action_result) => {
                            process_successful_action_result(&action_result, agent, &conversation_file_path, &memory, related_memories, response).await
                        },
                        Err(raw_result) => {
                            print_error_action_result(&raw_result);
                            let system_error = Chat { text: format!("{:?}", raw_result), whom: Whom::System };
                            add_chat_to_conversation(&conversation_file_path, system_error).await?;
                            Ok(LoopState::GetAIResponse)
                        }
                    }
                }
            }
        },
        Err((raw_response, error)) => {
            print_response_parse_error(&raw_response, &error);
            add_chat_to_conversation(&conversation_file_path, Chat { text: format!("{:?}", error), whom: Whom::System }).await?;
            Ok(LoopState::GetAIResponse)
        }
    }
}

// Assume memories have already been gathered.
async fn get_ai_response(agent: &Box<dyn Agent>, conversation: &Conversation, related_memories:  &Vec<MemoryData>, unparsed_ai_response: &mut Chat) -> Result<LoopState, Error> {
    // Conversation
    *unparsed_ai_response = agent.respond(conversation, related_memories).await?;
    Ok(LoopState::TakeAction)
}

async fn initialize_loop(agent: &Box<dyn Agent>, application_configuration: &ApplicationConfiguration, conversation: &Conversation, memory: &Box<dyn Memory>, related_memories: &mut Vec<MemoryData>, unparsed_ai_response: &mut Chat) -> Result<LoopState, Error> {
    match conversation.latest_chat_whom() {
        Some(whom) => {
            let latest_chat = conversation.conversation.last().unwrap();
            match whom {
                Whom::Agent => {
                    let parsed_response = latest_chat.parse();
                    match parsed_response {
                        Ok(_) => {
                            *unparsed_ai_response = latest_chat.clone();
                            Ok(LoopState::TakeAction)
                        }, 
                        Err((chat_text, error)) => {
                            *unparsed_ai_response = Chat { text: chat_text, whom: Whom::Agent };
                            let error_as_chat = Chat {text: format!{"{:?}", error}, whom: Whom::System };
                            add_chat_to_conversation(&application_configuration.system.conversation_file_path, error_as_chat).await?;
                            Ok(LoopState::GetAIResponse)
                        } 
                    }
                },
                Whom::System => {
                    // Assumption is that if the last message is a system message,
                    // then it is the ActionResult and we can just get ai to response.
                    // Memories should be based on AI Response (from right before ActionResult).
                    let last_ai_response = conversation.second_to_last_chat().unwrap();
                    let last_ai_response_as_memory = last_ai_response.as_memory_data(agent).await?;
                    *related_memories = memory.get_similar_memories(last_ai_response_as_memory).await?;
                    Ok(LoopState::GetAIResponse)
                }
                _ => {
                    let latest_chat_as_memory_data = latest_chat.as_memory_data(agent).await?;
                    *related_memories = memory.get_similar_memories(latest_chat_as_memory_data).await?;
                    Ok(LoopState::GetAIResponse)
                },
            }
        },
        None => Ok(LoopState::Exit)
    }
}

fn print_response_parse_error(response: &str, error: &Error) {
    println!("{}{}Unparseable Response: {}{}", style::Bold, color::Fg(color::Red), style::Reset, response);
    println!("{}{}Parsing Error: {}{:?}", style::Bold, color::Fg(color::Red), style::Reset, error);
}

fn print_error_action_result(error: &Error) {
    println!("{}{}Action Error: {}{:?}",style::Bold, color::Fg(color::Red), style::Reset, error);
}

async fn process_successful_action_result(action_result: &ActionResult, agent: &Box<dyn Agent>, conversation_file_path: &str, memory: &Box<dyn Memory>, related_memories: &mut Vec<MemoryData>, response: Response) -> Result<LoopState, Error> {
    action_result.print();
    println!("");
    let result_chat = Chat { text: action_result.to_variant_string(), whom: Whom::System };

    let (_,_) = tokio::join!(
        async {
            let previous_response_as_embedding = response.as_embedding(&agent).await?;
            let previous_response_as_memory = MemoryData(previous_response_as_embedding, response.as_one_string());
            *related_memories = memory.get_similar_memories(previous_response_as_memory).await?;
            Ok::<(), Error>(())
        },
        async {
            add_chat_to_conversation(conversation_file_path, result_chat).await
        }
    );

    Ok(LoopState::GetAIResponse)
}