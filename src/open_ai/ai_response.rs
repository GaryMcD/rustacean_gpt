use termion::{color, style};
use crate::action::Action;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiResponse {
    pub thoughts: String,
    pub reasoning: String,
    pub plan_of_action: String,
    pub constructive_criticism: String,
    pub next_command: Action
}

impl AiResponse {
    pub fn print(&self) {
        println!("{}{}{}Current Thoughts{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.thoughts, color::Fg(color::Reset));
        println!("{}{}{}Current Action Plan{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Green), style::Reset, color::Fg(color::LightGreen), self.plan_of_action, color::Fg(color::Reset));
        println!("{}{}{}Reasoning{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Magenta), style::Reset, color::Fg(color::LightMagenta), self.reasoning, color::Fg(color::Reset));
        println!("{}{}{}Self Criticism{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Magenta), style::Reset, color::Fg(color::LightMagenta), self.constructive_criticism, color::Fg(color::Reset));
        print!("{}{}{}Command To Run{}{}: ", style::Underline, style::Bold, color::Fg(color::Yellow), color::Fg(color::Reset), style::Reset);
        self.next_command.print();
        println!();
    }
}