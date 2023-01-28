use crate::{app_context::AppContext, domain::adapters::{Store, Git, CommitMsgStatus}, utils::string::OptionStr};

use super::Commit;

pub fn handler<G: Git, S: Store>(context: &AppContext<G, S>, commit: Commit) -> anyhow::Result<String> {
    let config = context.config.get_template_config(&commit.template)?;

    let branch = context
        .store
        .get_branch(
            &context.git.branch_name()?,
            &context.git.repository_name()?,
        )
        .ok();

    let contents = commit.commit_message(config.content.clone(), branch)?;

    let template_file = context.git.template_file_path()?;
    std::fs::write(&template_file, &contents)?;

    // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
    // Otherwise git will just abort the commit if theres no difference / change from the template.
    let commit_msg_complete = match commit.message.none_if_empty() {
        Some(_) => CommitMsgStatus::Completed,
        None => CommitMsgStatus::InComplete,
    };

    context
        .git
        .commit_with_template(&template_file, commit_msg_complete)?;

    Ok(contents)
}