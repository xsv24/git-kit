use std::fmt::Display;

use anyhow::Ok;
use colored::Colorize;
use inquire::{
    ui::{Color, RenderConfig, Styled},
    Select,
};

pub struct SelectItem {
    pub name: String,
    pub description: Option<String>,
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.name.green(),
            self.description.clone().unwrap_or_default().italic()
        )
    }
}

pub trait SelectorPrompt {
    fn prompt(&self, question: &str, options: Vec<SelectItem>) -> anyhow::Result<SelectItem>;
}

struct SelectPrompt;

impl SelectPrompt {
    fn get_render_config() -> RenderConfig {
        // inquire::set_global_render_config(get_render_config());
        RenderConfig {
            highlighted_option_prefix: Styled::new("➜").with_fg(Color::LightBlue),
            selected_checkbox: Styled::new("✅").with_fg(Color::LightGreen),
            unselected_checkbox: Styled::new("🔳"),
            ..RenderConfig::default()
        }
    }
}

impl SelectorPrompt for SelectPrompt {
    fn prompt(&self, question: &str, options: Vec<SelectItem>) -> anyhow::Result<SelectItem> {
        let len = options.len();
        let select: Select<SelectItem> = Select {
            message: question,
            options,
            help_message: Select::<SelectItem>::DEFAULT_HELP_MESSAGE,
            page_size: len,
            vim_mode: Select::<SelectItem>::DEFAULT_VIM_MODE,
            starting_cursor: Select::<SelectItem>::DEFAULT_STARTING_CURSOR,
            filter: Select::DEFAULT_FILTER,
            formatter: Select::DEFAULT_FORMATTER,
            render_config: Self::get_render_config(),
        };

        Ok(select.prompt()?)
    }
}

pub struct SelectorFactory;

impl SelectorFactory {
    pub fn create() -> Box<dyn SelectorPrompt> {
        Box::new(SelectPrompt)
    }
}
