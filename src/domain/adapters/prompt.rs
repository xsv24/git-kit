pub struct SelectItem {
    pub name: String,
    pub description: Option<String>,
}

pub trait Prompter {
    fn select(&self, question: &str, options: Vec<SelectItem>) -> anyhow::Result<SelectItem>;
}
