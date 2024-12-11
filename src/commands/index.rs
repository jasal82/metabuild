use anyhow::Error;
use colored::*;
use itertools::Itertools;
use metabuild_resolver::index::{Entry, Index};

pub fn add_git(index: &mut Index, name: &str, url: &str) -> Result<(), Error> {
    index.add_entry(name, Entry::Git { url: url.to_string() })
}

pub fn add_artifactory(index: &mut Index, name: &str, server: &str, repo: &str, path: &str) -> Result<(), Error> {
    index.add_entry(name, Entry::Artifactory {
        server: server.to_string(),
        repo: repo.to_string(),
        path: path.to_string(),
    })
}

pub fn remove(index: &mut Index, name: &str) -> Result<(), Error> {
    index.remove_entry(name)
}

pub fn revert(index: &mut Index) -> Result<(), Error> {
    index.revert()
}

pub fn push(index: &Index) -> Result<(), Error> {
    index.push()
}

pub fn list(index: &Index) -> Result<(), Error> {
    if let Ok(entries) = index.get_entries() {
        for entry in entries.iter().sorted() {
            match index.get_entry(entry) {
                Ok(Entry::Git { url }) => println!("{}\n  {} {}", entry.bright_green().bold(), "(Git)".bright_yellow(), url),
                Ok(Entry::Artifactory { server, repo, path }) => println!("{}\n  {} {}/{}/{}", entry.bright_green().bold(), "(Artifactory)".bright_yellow(), server, repo, path),
                Err(e) => eprintln!("{}: {}", "Error".red().bold(), e),
            }
        }
    }
    Ok(())
}
