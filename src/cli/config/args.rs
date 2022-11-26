#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Add / register a custom config file.
    Add { key: String, path: String },
    /// Select config file to use.
    Set { key: String },
    /// Display the current config in use.
    Show,
    /// Reset to the default config.
    Reset,
}
