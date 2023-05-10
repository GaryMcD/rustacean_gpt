// src/agent/response.rs

use anyhow::Error;
use crate::{memory::Embedding, system::Action};
use serde::{Deserialize, Serialize};
use super::Agent;
use termion::{color, style};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub action_plan: Vec<String>,
    pub next_command: Action,
    pub reasoning: String,
    pub constructive_criticism: String,
    pub thoughts: String,
}

impl Response {
    pub async fn as_embedding(&self, agent: &Box<dyn Agent>) -> Result<Embedding, Error> {
        agent.get_string_embedding(&self.as_one_string()).await
    }

    pub fn as_one_string(&self) -> String {
        format!(
            "Most Recent Memory: THOUGHTS: {}. REASONING: {}. ACTION PLAN: {}. NEXT COMMAND: {}. CONSTRUCTIVE CRITICISM: {}", 
            self.thoughts, 
            self.reasoning, 
            self.action_plan.join(" "), 
            self.next_command.to_variant_string(), 
            self.constructive_criticism)
    }

    pub fn print(&self) {
        println!("{}{}{}Current Thoughts{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.thoughts, color::Fg(color::Reset));
        println!("{}{}{}Current Action Plan{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Green), style::Reset, color::Fg(color::LightGreen), self.action_plan.join(" "), color::Fg(color::Reset));
        println!("{}{}{}Reasoning{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Magenta), style::Reset, color::Fg(color::LightMagenta), self.reasoning, color::Fg(color::Reset));
        println!("{}{}{}Self Criticism{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Magenta), style::Reset, color::Fg(color::LightMagenta), self.constructive_criticism, color::Fg(color::Reset));
        print!("{}{}{}Command To Run{}{}: ", style::Underline, style::Bold, color::Fg(color::Yellow), color::Fg(color::Reset), style::Reset);
        self.next_command.print();
        println!();
    }
}