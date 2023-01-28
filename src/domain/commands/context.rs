use crate::{domain::{adapters::{Store, Git}, models::Branch}, app_context::AppContext};

#[derive(Debug, Clone)]
pub struct Context {
    /// Issue ticket number related to the current branch.
    pub ticket: Option<String>,
    /// Short describing a section of the codebase the changes relate to.
    pub scope: Option<String>,
    /// Issue ticket number link.
    pub link: Option<String>,
}

pub fn handler<G: Git, S: Store>(context: &AppContext<G, S>, args: super::Context) -> anyhow::Result<Branch> {
    // We want to store the branch name against and ticket number
    // So whenever we commit we get the ticket number from the branch
    let repo_name = context.git.repository_name()?;
    let branch_name = context.git.branch_name()?;

    let branch = Branch::new(&branch_name, &repo_name, args.ticket, args.link, args.scope)?;
    context.store.persist_branch(&branch)?;

    Ok(branch)
}