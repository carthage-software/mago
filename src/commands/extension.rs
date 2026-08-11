//! Inspect and validate configured external extension hosts.

use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use crate::config::Configuration;
use crate::error::Error;
use crate::extensions::initialize_external_linter;

/// Manage external extensions configured for this workspace.
#[derive(Parser, Debug)]
#[command(name = "extension", about = "Inspect and validate external extensions.")]
pub struct ExtensionCommand {
    #[command(subcommand)]
    command: ExtensionSubcommand,
}

#[derive(Subcommand, Debug)]
enum ExtensionSubcommand {
    /// List configured extensions and the linter rules they expose.
    List {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Start every configured host and validate its registration.
    Validate,
}

impl ExtensionCommand {
    pub fn execute(self, configuration: Configuration) -> Result<ExitCode, Error> {
        let mago_threads = configuration.threads;
        let enabled_hosts = configuration.extension_hosts.iter().filter(|(_, host)| host.enabled).collect::<Vec<_>>();
        let external = initialize_external_linter(
            &configuration.extension_hosts,
            configuration.php_version,
            configuration.threads,
        )
        .map_err(mago_orchestrator::OrchestratorError::from)?;

        match self.command {
            ExtensionSubcommand::List { json } => {
                if json {
                    let extensions = external.as_ref().map_or(&[][..], |external| external.extensions());
                    let hosts = enabled_hosts.iter().map(|(host, host_configuration)| {
                        serde_json::json!({
                            "host": host,
                            "adaptive": host_configuration.workers == 0,
                            "workers": host_configuration.worker_count(mago_threads).get(),
                        })
                    });
                    let extensions = extensions.iter().map(|extension| {
                        serde_json::json!({
                            "identifier": extension.identifier,
                            "name": extension.name,
                            "version": extension.version,
                            "linter-rules": extension.rules.iter().map(|rule| serde_json::json!({
                                "code": rule.code,
                                "name": rule.name,
                                "description": rule.description,
                                "default-level": rule.default_level,
                                "default-enabled": rule.default_enabled,
                                "targets": rule.targets.iter().map(ToString::to_string).collect::<Vec<_>>(),
                            })).collect::<Vec<_>>(),
                        })
                    });
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "hosts": hosts.collect::<Vec<_>>(),
                            "extensions": extensions.collect::<Vec<_>>(),
                        }))?
                    );
                } else if let Some(external) = external {
                    println!("Extension hosts:");
                    for (host, host_configuration) in &enabled_hosts {
                        let workers = host_configuration.worker_count(mago_threads);
                        if host_configuration.workers == 0 {
                            println!("  {host} (adaptive, up to {workers} workers)");
                        } else {
                            println!("  {host} ({workers} workers)");
                        }
                    }

                    println!("Registered extensions:");
                    for extension in external.extensions() {
                        println!("{} ({})", extension.name, extension.identifier);
                        println!("  Version: {}", extension.version);
                        println!("  Linter rules: {}", extension.rules.len());
                        for rule in &extension.rules {
                            println!("    {} ({})", rule.code, rule.default_level);
                        }
                    }
                } else {
                    println!("No external extensions are configured.");
                }
            }
            ExtensionSubcommand::Validate => {
                if let Some(external) = external {
                    let extensions = external.extensions().len();
                    println!("Validated {extensions} extension(s) from {} host(s).", enabled_hosts.len());
                } else {
                    println!("No external extensions are configured.");
                }
            }
        }

        Ok(ExitCode::SUCCESS)
    }
}
