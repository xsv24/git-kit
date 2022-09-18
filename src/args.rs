use crate::{branch::Branch, git_commands::{GitCommands}, context::Context};
use clap::Args;

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct Arguments {
    /// Issue ticket number related to the commit.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,
}

impl Arguments {
    pub fn commit_message<C: GitCommands>(&self, template: String, context: &Context<C>) -> anyhow::Result<String> {
        let ticket_num = match &self.ticket {
            Some(num) => num.into(),
            None => { 
                let branch = Branch::get(
                    &context.commands.get_branch_name()?,
                    &context.commands.get_repo_name()?,
                    &context
                )?;

                branch.ticket
            },
        };

        let contents = template.replace("{ticket_num}", &format!("[{}]", ticket_num));

        let contents = match &self.message {
            Some(message) => contents.replace("{message}", &message),
            None => contents.replace("{message}", ""),
        };

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::git_commands::Git;

    use super::*;

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let project_dir = fake_project_dir()?;

        let context = Context {
            connection: setup_db(None)?,
            project_dir,
            commands: Git { }
        };

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let project_dir = fake_project_dir()?;

        let context = Context {
            connection: setup_db(None)?,
            project_dir,
            commands: Git { }
        };

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: None,
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] ", args.ticket.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let project_dir= fake_project_dir()?;
        let commands = Git { };
        let name = commands.get_branch_name()?;
        let repo = commands.get_repo_name()?;

        let branch = Branch::new(&name, &repo, None)?;

        let context = Context {
            connection: setup_db(Some(&branch))?,
            project_dir,
            commands 
        };

        let args = Arguments {
            ticket: None,
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] {}", name, args.message.unwrap());

        context.close()?;

        assert_eq!(actual, expected);

        Ok(())
    }

    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .expect("Failed to retrieve 'git-kit' config");

        Ok(dirs)
    }

    fn setup_db(branch: Option<&Branch>) -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            )",
            (),
        )?;

        if let Some(branch) = branch {
            conn.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    branch.created.to_rfc3339(),
                ),
            )?;
        }

        Ok(conn)
    }
}
