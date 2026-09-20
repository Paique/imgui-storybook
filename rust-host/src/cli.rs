//! Minimal hand-rolled CLI parser. Rust mirror of `host/Cli.java` (same flags and defaults).

use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    List,
    Serve,
    Capture,
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub command: Command,
    pub json: bool,
    pub no_demo: bool,
    pub port: u16,
    pub width: i32,
    pub height: i32,
    pub out: String,
    pub themes: Vec<String>,
    pub scales: Vec<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            command: Command::List,
            json: false,
            no_demo: false,
            port: 8765,
            width: 900,
            height: 600,
            out: "captures".to_string(),
            themes: vec!["light".into(), "dark".into()],
            scales: vec!["1".into()],
        }
    }
}

impl Cli {
    pub fn parse(args: &[String]) -> Result<Cli> {
        let mut cli = Cli::default();
        let mut i = 0;
        while i < args.len() {
            let arg = args[i].as_str();
            match arg {
                "--list" => cli.command = Command::List,
                "--json" => cli.json = true,
                "--serve" => cli.command = Command::Serve,
                "--capture" => cli.command = Command::Capture,
                "--no-demo" => cli.no_demo = true,
                "--port" => cli.port = next(args, arg, &mut i)?.parse()?,
                "--width" => cli.width = next(args, arg, &mut i)?.parse()?,
                "--height" => cli.height = next(args, arg, &mut i)?.parse()?,
                "--out" => cli.out = next(args, arg, &mut i)?.to_string(),
                "--themes" => cli.themes = split(next(args, arg, &mut i)?),
                "--scales" => cli.scales = split(next(args, arg, &mut i)?),
                _ => bail!("unknown argument: {arg}"),
            }
            i += 1;
        }
        Ok(cli)
    }
}

fn next<'a>(args: &'a [String], flag: &str, i: &mut usize) -> Result<&'a str> {
    *i += 1;
    args.get(*i).map(|s| s.as_str()).ok_or_else(|| anyhow::anyhow!("{flag} expects a value"))
}

fn split(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect()
}

pub fn usage() -> &'static str {
    "imgui-storybook rust host

Usage:
  host --list --json
  host --serve [--port 8765] [--width 900] [--height 600] [--no-demo]
  host --capture [--out DIR] [--themes light,dark] [--scales 1,2] \
       [--width 900] [--height 600] [--no-demo]
"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults() {
        let cli = Cli::parse(&args(&[])).unwrap();
        assert_eq!(cli.command, Command::List);
        assert_eq!(cli.port, 8765);
        assert_eq!(cli.width, 900);
        assert_eq!(cli.height, 600);
        assert_eq!(cli.out, "captures");
        assert_eq!(cli.themes, vec!["light", "dark"]);
        assert_eq!(cli.scales, vec!["1"]);
    }

    #[test]
    fn full_parse() {
        let cli = Cli::parse(&args(&[
            "--serve",
            "--port",
            "9000",
            "--width",
            "800",
            "--height",
            "500",
            "--no-demo",
        ]))
        .unwrap();
        assert_eq!(cli.command, Command::Serve);
        assert_eq!(cli.port, 9000);
        assert_eq!(cli.width, 800);
        assert_eq!(cli.height, 500);
        assert!(cli.no_demo);
    }

    #[test]
    fn capture_flags() {
        let cli =
            Cli::parse(&args(&["--capture", "--out", "out/x", "--themes", "Dark, LIGHT", "--scales", "1,2"]))
                .unwrap();
        assert_eq!(cli.command, Command::Capture);
        assert_eq!(cli.out, "out/x");
        assert_eq!(cli.themes, vec!["dark", "light"]);
        assert_eq!(cli.scales, vec!["1", "2"]);
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(Cli::parse(&args(&["--wat"])).is_err());
        assert!(Cli::parse(&args(&["--port"])).is_err());
    }
}
