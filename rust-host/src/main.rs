//! Entry point — Rust mirror of `host/HostMain.java`.

use imgui_storybook_rust_host::{catalog, capture, cli, demo, info, registry, serve};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("{}", cli::usage());
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> anyhow::Result<()> {
    let cli = cli::Cli::parse(args)?;
    let mut registry = registry::StoryRegistry::new();
    if !cli.no_demo {
        demo::register_all(&mut registry);
    }
    match cli.command {
        cli::Command::List => {
            println!(
                "{}",
                catalog::to_json_pretty(&registry, info::host_version(), info::binding_version())
            );
        }
        cli::Command::Capture => capture::run(&cli, &mut registry)?,
        cli::Command::Serve => serve::run(&cli, &mut registry)?,
    }
    Ok(())
}
