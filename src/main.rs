use std::path::Path;
use std::process::Command;

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use url::Url;

const DEFAULT_BASE_PATH: &'static str = "~/code";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Base path for repos
    #[arg(long, default_value = DEFAULT_BASE_PATH)]
    base_path: String,

    /// Type of repo
    r#type: String,

    /// URL or path of target repo
    repo: String,
}

fn parse_host(s: &str) -> Option<String> {
    let Ok(url) = Url::parse(s) else {
        return None;
    };

    Some(
        format!("{}{}", url.host_str().unwrap_or_default(), url.path())
            .trim_start_matches('/')
            .to_string(),
    )
}

fn parse_ssh(s: &str) -> Option<String> {
    let re = Regex::new(r"(?:.+@)?(\w[\w\-\.]+):(.+)").unwrap();

    let Some(matches) = re.captures(s) else {
        return None;
    };

    Some(
        format!("{}/{}", &matches[1], &matches[2].trim_start_matches('/'))
            .trim_start_matches('/')
            .to_string(),
    )
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path = parse_host(&args.repo)
        .or_else(|| parse_ssh(&args.repo))
        .unwrap_or(args.repo.clone());

    let target = Path::new(&args.base_path)
        .join(args.r#type)
        .join(path.trim_end_matches(".git"));

    Command::new("git")
        .args(["clone", &args.repo, &target.to_string_lossy()])
        .spawn()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_host_parsing() {
        assert_eq!(parse_host("file:///foo/bar"), Some("foo/bar".to_string()));
        assert_eq!(parse_host("http:/foo/bar"), Some("foo/bar".to_string()));
        assert_eq!(parse_host("http://foo/bar"), Some("foo/bar".to_string()));

        for s in &["git@github.com:/foo/bar.git", "foo", "foo/bar", "/foo/bar"] {
            println!("{}", s);
            assert!(parse_host(s).is_none());
        }
    }

    #[test]
    fn test_ssh_parsing() {
        assert_eq!(parse_ssh("foo:bar.git"), Some("foo/bar.git".to_string()));
        assert_eq!(
            parse_ssh("github.com:foo/bar.git"),
            Some("github.com/foo/bar.git".to_string())
        );
        assert_eq!(
            parse_ssh("git@github.com:/foo/bar.git"),
            Some("github.com/foo/bar.git".to_string())
        );

        for s in &["foo", "foo/bar", "/foo/bar"] {
            println!("{}", s);
            assert!(parse_ssh(s).is_none());
        }
    }
}
