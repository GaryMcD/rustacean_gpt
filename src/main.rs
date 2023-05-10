// src/main.rs

mod agent;
mod configuration;
mod memory;
mod system;
mod user;

use agent::{Agent, openai::GPT};
use anyhow::Error;
use configuration::{AgentConfiguration, ApplicationConfiguration, get_initial_prompt, MemoryConfiguration};
use inquire::Text;
use memory::{Memory, Pinecone};
use system::{add_chats_to_conversation, application_loop, Chat, conversation, Conversation, Whom};
use tokio;


const CONFIGURATION_FILE_PATH: &str = "./config/configuration.json";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (
        application_configuration, 
        agent, 
        mut memory
    ) = initialize().await?;

    application_loop(&application_configuration, agent, &mut memory).await?;

    Ok(())
}

async fn get_conversation(application_configuration: &ApplicationConfiguration) -> Result<Conversation, Error> {
    let conversation_file_path = &application_configuration.system.conversation_file_path;
    match conversation(conversation_file_path).await? {
        Some(conversation) => Ok(conversation),
        None => {
            let initial_prompt_getter = get_initial_prompt(&application_configuration.system.initial_prompt);
            let objective_getter = get_objective();

            let (initial_prompt, objective) = tokio::join!(initial_prompt_getter, objective_getter);
            let chats = vec![
                Chat { text: initial_prompt?, whom: Whom::System },
                Chat { text: objective?, whom: Whom::User }
            ];

            add_chats_to_conversation(conversation_file_path, chats).await
        }
    }
}

async fn get_objective() -> Result<String, Error> {
    let objective_prompt = Text::new("What is your objective for Rustacean-GPT?").prompt();
    match objective_prompt {
        Ok(objective) => Ok(format!("Your Objective: {}", objective)),
        Err(e) => Err(Error::from(e))
    }
}

async fn initialize() -> Result<(ApplicationConfiguration, Box<dyn Agent>, Box<dyn Memory>), Error> {
    let application_configuration = load_configuration().await?;

    let agent_init = initialize_agent(&application_configuration.agent);
    let memory_init = initialize_memory(&application_configuration.memory);
    let conversation_getter = get_conversation(&application_configuration);

    let (agent, memory, conversation) = tokio::join!(agent_init, memory_init, conversation_getter);
    let agent = agent?;
    let memory = memory?;
    let _ = conversation?;

    Ok((application_configuration, agent, memory))
}

async fn initialize_agent(agent_condiguration: &AgentConfiguration) -> Result<Box<dyn Agent>, Error> {
    match agent_condiguration {
        AgentConfiguration::OpenAIAgentConfiguration(openai_agent_configuration) => {
            let mut agent = GPT::new(openai_agent_configuration);
            agent.initialize().await?;
            Ok(agent)
        }
    }
}

async fn initialize_memory(memory_configuration: &MemoryConfiguration) -> Result<Box<dyn Memory>, Error> {
    match memory_configuration {
        MemoryConfiguration::PineconeConfiguration(pinecone_memory_configuration) => {
            let mut memory = Pinecone::new(pinecone_memory_configuration);
            memory.initialize().await?;
            Ok(memory)
        }
    }
}

async fn load_configuration() -> Result<ApplicationConfiguration, Error> {
    match configuration::load_configuration(CONFIGURATION_FILE_PATH).await {
        Ok(config) => Ok(config),
        Err(e) => Err(Error::from(e)),
    }
}