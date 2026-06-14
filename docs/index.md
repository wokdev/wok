# Git Wok

A powerful multirepo management tool built with Rust.

---

Git Wok manages multiple Git repositories as one workspace, using Git
submodules as the source of truth.

Use it to keep many component repos in sync for daily development, updates,
and releases from a single umbrella repository.

## Key Features

- Workspace setup for existing umbrellas (`init`) or directories of repos
  (`assemble`)
- Daily multirepo operations (`status`, `switch`, `lock`, `update`, `push`)
- Repo configuration management (`add`, `rm`)
- Release workflows across repos (`tag create`, `tag list`, `tag push`)
- Selective targeting with current-branch defaults, `--all`, and explicit paths
- Per-repo `skip_for` controls in `wok.toml` for bulk commands
- Git LFS-aware `update` and `push`
- Shell completion support for Bash, Zsh, and Fish
- Authentication diagnostics with `test-auth`
- Agent Skills for AI-assisted multirepo workflows

[Get started](getting-started.md){ .md-button .md-button--primary }
[View Commands](cli.md){ .md-button }
[Wokfile Reference](wokfile.md){ .md-button }

## Community

Need help or want to share feedback? Join the [Git Wok group on Delta Chat](https://i.delta.chat/#667BD2FB6B122F4138F29A17861B4E257DCDFDB9&a=lig%40countzero.co&g=Git%20Wok&x=0FgEK_cMRZ6NMvG1PAekdJE3&i=9Jn9KZM9tErF-O0k8xvadsn_&s=DyV77Vq3p4y86HX9rRuOMvm2).
