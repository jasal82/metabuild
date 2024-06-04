mod index;
pub mod inventory;
mod module;
mod repository;

use inventory::Inventory;
use itertools::Itertools;
use module::VersionReq;
use resolvo::{DefaultSolvableDisplay, Solver, UnsolvableOrCancelled};
use std::collections::HashMap;

#[derive(Debug)]
pub enum SolverError {
    Unsolvable(String),
    Cancelled,
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SolverError::Unsolvable(_) => write!(f, "Unsolvable"),
            SolverError::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

impl std::error::Error for SolverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SolverError::Unsolvable(_) => None,
            SolverError::Cancelled => None,
        }
    }
}

pub fn solve(
    inventory: &Inventory,
    dependencies: HashMap<String, semver::VersionReq>,
) -> Result<HashMap<String, semver::Version>, SolverError> {
    let mapped_dependencies = dependencies
        .iter()
        .map(|d| inventory.map_dependency(d.0, &VersionReq { 0: d.1.clone() }))
        .collect_vec();
    let mut solver = Solver::new(inventory);
    match solver.solve(mapped_dependencies) {
        Ok(result) => Ok(result
            .iter()
            .map(|s| inventory.map_solvable(s))
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()),
        Err(UnsolvableOrCancelled::Unsolvable(problem)) => {
            let reason = problem
                .display_user_friendly(&solver, inventory.pool(), &DefaultSolvableDisplay)
                .to_string();
            Err(SolverError::Unsolvable(reason))
        }
        Err(UnsolvableOrCancelled::Cancelled(_)) => Err(SolverError::Cancelled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solvable() -> Result<(), Error> {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut inventory = Inventory::new("https://github.com/jasal82/index.git", temp_dir.path());
        inventory.update_cache()?;
        let mut dependencies: HashMap<String, semver::VersionReq> = HashMap::new();
        dependencies.insert("module1".to_string(), semver::VersionReq::parse("^1")?);
        dependencies.insert("module2".to_string(), semver::VersionReq::parse("^2")?);
        let result = solve(&inventory, dependencies)?;
        assert!(
            result.contains_key("module1")
                && result
                    .get("module1")
                    .unwrap()
                    .eq(&semver::Version::parse("1.0.1")?)
        );
        assert!(
            result.contains_key("module2")
                && result
                    .get("module2")
                    .unwrap()
                    .eq(&semver::Version::parse("2.4.0")?)
        );
        Ok(())
    }

    #[test]
    fn test_unsolvable() -> Result<(), Error> {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut inventory = Inventory::new("https://github.com/jasal82/index.git", temp_dir.path());
        inventory.update_cache()?;
        let mut dependencies: HashMap<String, semver::VersionReq> = HashMap::new();
        dependencies.insert("module1".to_string(), semver::VersionReq::parse("^1")?);
        dependencies.insert("module2".to_string(), semver::VersionReq::parse("~2.3.0")?);
        let result = solve(&inventory, dependencies);
        assert!(result.is_err());
        match result.err().unwrap() {
            SolverError::Unsolvable(reason) => {
                println!("{}", reason);
            }
            _ => {}
        }
        Ok(())
    }
}
