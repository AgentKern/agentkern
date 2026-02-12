//! AgentKern-Gate Policy CLI
//!
//! Standalone tool for managing the security policy registry.

use agentkern_gate::policy::registry::{PolicyRegistry, PolicyBundle, PolicyMetadata, PolicyCategory};
use agentkern_gate::Policy;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;

#[derive(Parser)]
#[command(name = "gate-cli")]
#[command(about = "AgentKern-Gate Policy Management CLI", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "./policies")]
    policy_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registered policies
    List,
    /// Export a policy bundle to a file
    Export {
        /// Policy ID
        id: String,
        /// Output file path (default: <id>.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Import a policy bundle from a file
    Import {
        /// Path to the policy bundle file (.json or .yaml)
        file: PathBuf,
        /// Override the author in the metadata
        #[arg(short, long)]
        author: Option<String>,
    },
    /// Create a new local policy template
    Init {
        /// Policy ID to create
        id: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut registry = PolicyRegistry::new(&cli.policy_dir)
        .map_err(|e| anyhow::anyhow!("Failed to initialize policy registry: {}", e))?;
    
    // Attempt to load existing policies
    let _ = registry.load_all();

    match cli.command {
        Commands::List => {
            let policies = registry.list_policies();
            println!("\n🛡️  AgentKern Policy Registry: {}", cli.policy_dir.display());
            println!("{:-<60}", "");
            println!("{:<20} | {:<10} | {:<10} | {:<15}", "ID", "Version", "Category", "Author");
            println!("{:-<60}", "");
            
            for bundle in policies {
                println!(
                    "{:<20} | {:<10} | {:<10?} | {:<15}",
                    bundle.policy.id,
                    bundle.metadata.version,
                    bundle.metadata.category,
                    bundle.metadata.author
                );
            }
            println!("{:-<60}", "");
            println!("Total: {} policies", registry.list_policies().len());
        }
        Commands::Export { id, output } => {
            let export_data = registry.export_bundle(&id)
                .map_err(|e| anyhow::anyhow!("Export failed: {}", e))?;
            
            let out_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.json", id)));
            fs::write(&out_path, export_data)?;
            println!("✅ Policy '{}' exported to {}", id, out_path.display());
        }
        Commands::Import { file, author } => {
            let content = fs::read_to_string(&file)?;
            let mut bundle: PolicyBundle = if file.extension().is_some_and(|ext| ext == "json") {
                serde_json::from_str(&content)?
            } else {
                serde_yaml::from_str(&content)?
            };
            
            if let Some(new_author) = author {
                bundle.metadata.author = new_author;
            }
            
            let id = bundle.policy.id.clone();
            registry.save_bundle(bundle)
                .map_err(|e| anyhow::anyhow!("Import failed: {}", e))?;
            
            println!("✅ Successfully imported policy '{}' from {}", id, file.display());
        }
        Commands::Init { id } => {
            use agentkern_gate::PolicyRule;
            use agentkern_gate::policy::PolicyAction;

            let bundle = PolicyBundle {
                metadata: PolicyMetadata {
                    author: "Local Developer".into(),
                    version: "0.1.0".into(),
                    tags: vec!["local".into()],
                    category: PolicyCategory::Community,
                },
                policy: Policy {
                    id: id.clone(),
                    name: format!("New Policy {}", id),
                    description: "Replace with your description".into(),
                    priority: 10,
                    enabled: true,
                    jurisdictions: vec![],
                    namespace: "default".into(),
                    rules: vec![PolicyRule {
                        id: format!("{}-rule-1", id),
                        condition: "context.amount < 100".into(),
                        action: PolicyAction::Allow,
                        message: Some("Default small-amount allow".into()),
                        risk_score: Some(0),
                    }],
                },
            };
            
            registry.save_bundle(bundle)
                .map_err(|e| anyhow::anyhow!("Init failed: {}", e))?;
            
            println!("✅ Initialized new policy '{}' in {}", id, cli.policy_dir.display());
        }
    }

    Ok(())
}
