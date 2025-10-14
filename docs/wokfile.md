# Wokfile (`wok.yaml`)

Git Wok stores the configuration for sub repos in a file.

The file uses YAML syntax.

The file should be stored in the main repo containing all sub repos.

The default path of the file is `wok.yaml` relative to the root dir of the main repo.

Git Wok will try to find the `wok.yaml` file using alternative paths in the following order:

- `wok.yaml` (the default)
- `wok.yml`
- `Wokfile`


## Wok.yaml Syntax


### First-level Keys

- `version` -- String -- Wokfile Specification version. Must contain "1.0" for now.
- `repos` -- List of Repo Objects -- Each object in the list corresponds to a configured sub repo.


### Repo Object

- `path` -- String -- path to the configured submodule
- `ref` -- String -- current branch configured for the `HEAD` reference of the submodule to point to
- `skip_for` -- List of Strings (optional) -- command names that should skip this repo when `--all` is used. The repo is still processed if it is explicitly listed. Currently honoured by `tag`, `push`, `switch`, and `update`.
