use anyhow::*;
use std::io::Write;
use std::result::Result::Ok;

use crate::repo;

pub fn test_auth<W: Write>(repo: &repo::Repo, stdout: &mut W) -> Result<()> {
    writeln!(stdout, "Testing authentication for repository...")?;
    writeln!(stdout, "Repository: {}", repo.work_dir.display())?;
    writeln!(stdout)?;

    // Get current branch
    let head_ref = repo.git_repo.head()?;
    let branch_name = head_ref
        .shorthand()
        .ok_or_else(|| anyhow!("Cannot get branch name"))?;

    writeln!(stdout, "Current branch: {}", branch_name)?;

    // Get tracking branch
    if let Some(tracking) = repo.tracking_branch(branch_name)? {
        writeln!(stdout, "Remote: {}", tracking.remote)?;
        writeln!(stdout, "Remote ref: {}", tracking.remote_ref)?;
        writeln!(stdout)?;

        // Try to find the remote
        match repo.git_repo.find_remote(&tracking.remote) {
            Ok(mut remote) => {
                writeln!(stdout, "Attempting to connect to remote...")?;
                writeln!(stdout)?;

                // Try to connect
                match remote.connect_auth(
                    git2::Direction::Fetch,
                    Some(repo.remote_callbacks_verbose()?),
                    None,
                ) {
                    Ok(connection) => {
                        writeln!(stdout)?;
                        writeln!(stdout, "? Connection successful!")?;
                        writeln!(stdout)?;
                        writeln!(stdout, "Available remote heads:")?;
                        for head in connection.list()?.iter() {
                            writeln!(stdout, "  - {}", head.name())?;
                        }
                        drop(connection);
                    },
                    Err(e) => {
                        writeln!(stdout)?;
                        writeln!(stdout, "? Connection failed: {}", e)?;
                        writeln!(stdout)?;
                        writeln!(stdout, "Troubleshooting steps:")?;
                        writeln!(stdout, "1. Check SSH agent is running: ssh-add -l")?;
                        writeln!(
                            stdout,
                            "2. Verify SSH_AUTH_SOCK is set: echo $SSH_AUTH_SOCK"
                        )?;
                        writeln!(
                            stdout,
                            "3. Test SSH connection: ssh -T git@<hostname>"
                        )?;
                        writeln!(stdout, "4. Check SSH keys exist: ls -la ~/.ssh/")?;
                        return Err(e.into());
                    },
                }
            },
            Err(e) => {
                writeln!(stdout, "? Remote '{}' not found: {}", tracking.remote, e)?;
                return Err(e.into());
            },
        }
    } else {
        writeln!(
            stdout,
            "No tracking branch configured for '{}'",
            branch_name
        )?;
        writeln!(stdout)?;
        writeln!(stdout, "To set up tracking, run:")?;
        writeln!(
            stdout,
            "  git branch --set-upstream-to=<remote>/<branch> {}",
            branch_name
        )?;
    }

    Ok(())
}
