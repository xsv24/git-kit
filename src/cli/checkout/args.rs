use anyhow::Ok;
use clap::Args;

use crate::{domain::{adapters::prompt::Prompter, commands::checkout::Checkout}, utils::or_else_try::OrElseTry, entry::Interactive};

#[derive(Debug, Args, Clone)]
pub struct Arguments {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(&self, prompt: P, interactive: &Interactive) -> anyhow::Result<Checkout> {
        let domain = match interactive {
            Interactive::Enable => Checkout {
                name: self.name.clone(),
                ticket: self.ticket.clone().or_else_try(|| prompt.text("Ticket:"))?,
                scope: self.scope.clone().or_else_try(|| prompt.text("Scope:"))?,
                link: self.link.clone().or_else_try(|| prompt.text("Link:"))?,
            },
            Interactive::Disable => Checkout { 
                name: self.name.clone(),
                ticket: self.ticket.clone(),
                scope: self.scope.clone(),
                link: self.link.clone(),
            }
        };

        Ok(domain)
    }
}
