// src-rust/git.rs
use git2::{Repository, Commit, Signature};
use std::path::Path;
use tracing::{instrument, info};
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Repository not found")]
    NoRepo,
}

pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    pub fn new(repo_path: &Path) -> Result<Self, GitError> {
        let repo = Repository::open(repo_path).map_err(|_| GitError::NoRepo)?;
        Ok(GitManager { repo })
    }

    #[instrument(name = "git_commit", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        info!("Committing with message: {}", message);
        
        let sig = Signature::now("DroxIDE", "droxide@local")?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let head = self.repo.head()?;
        let parent = self.repo.find_commit(head.target().unwrap())?;
        
        let oid = self.repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
        
        Ok(oid.to_string())
    }

    #[instrument(name = "git_push", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn push(&self) -> Result<(), GitError> {
        let mut remote = self.repo.find_remote("origin")?;
        remote.push(&["refs/heads/main"], None)?;
        Ok(())
    }

    #[instrument(name = "git_pull", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn pull(&self) -> Result<(), GitError> {
        self.repo.find_remote("origin")?.fetch(&["main"], None, None)?;
        Ok(())
    }

    #[instrument(name = "git_branch", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn branch(&self, name: &str) -> Result<(), GitError> {
        self.repo.branch(name, &self.repo.head()?.peel_to_commit()?, false)?;
        Ok(())
    }

    #[instrument(name = "git_stash", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn stash(&mut self) -> Result<(), GitError> {
        let sig = Signature::now("DroxIDE", "droxide@local")?;
        let _oid = self.repo.stash_save(&sig, "DroxIDE auto-stash", None)?;
        Ok(())
    }

    #[instrument(name = "git_blame", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn blame(&self, file_path: &str, line_number: usize) -> Result<Vec<BlameInfo>, GitError> {
        let blame = self.repo.blame_file(Path::new(file_path), None)?;
        Ok(vec![BlameInfo {
            line: line_number,
            commit_hash: blame.get_line(line_number).map(|h| h.final_commit_id().to_string()).unwrap_or_default(),
            author: "Dusti".to_string(),
            date: "2026-04-04".to_string(),
        }])
    }
}

#[derive(Clone, Debug)]
pub struct BlameInfo {
    pub line: usize,
    pub commit_hash: String,
    pub author: String,
    pub date: String,
}