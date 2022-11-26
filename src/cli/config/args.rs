#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Update the global configuration
    Create { key: String, path: String },
    /// Set the configuration.
    Set { key: String },
    /// Print out the current configuration
    Show,
    /// Reset to the default configuration
    Reset,
}
