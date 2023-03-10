use crate::domain::{
    adapters::{CheckoutStatus, Git, Store},
    errors::{Errors, GitError},
    models::Branch,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    /// Name of the branch to checkout or create.
    pub name: String,
    /// Issue ticket number related to the branch.
    pub ticket: Option<String>,
    /// Short describing a section of the codebase the changes relate to.
    pub scope: Option<String>,
    /// Issue ticket number link.
    pub link: Option<String>,
}

pub fn handler<G: Git, S: Store>(git: &G, store: &S, args: Checkout) -> Result<Branch, Errors> {
    // Attempt to create branch
    let create = git.checkout(&args.name, CheckoutStatus::New);

    // If the branch already exists check it out
    if let Err(err) = create {
        log::error!("failed to create new branch: {}", err);

        git.checkout(&args.name, CheckoutStatus::Existing)
            .map_err(|_| Errors::Git(GitError::Write))?;
    }

    // We want to store the branch name against and ticket number
    // So whenever we commit we get the ticket number from the branch
    let repo_name = git
        .repository_name()
        .map_err(|_| Errors::Git(GitError::Read))?;

    let branch = Branch::new(&args.name, &repo_name, args.ticket, args.link, args.scope);
    store
        .persist_branch(&branch)
        .map_err(|_| Errors::PersistError)?;

    Ok(branch)
}
