# Wok Commands Reference

```sh
wok [<OPTIONS>]
```

## -f / --wok-file-path

```sh
wok -f <WOK_FILE_PATH>
wok --wok-file-path <WOK_FILE_PATH>
```

!!! abstract "Default"
    `wok.yaml`

Overrides default path to [Wokfile](./wokfile.md).


## --help

```sh
wok --help
```

Shows list of possible commands and global options reference. When used with a specific subcommand shows help for that subcommand.


## init

```sh
wok init
```

Initialize Wok configuration in an Umbrella Repo.

This creates [Wokfile](./wokfile.md) in the repo and introspects existing submodules adding them to the Wokfile, optionally switching them to the same branch as the current branch.

`wok init` fails if Wokfile is already present in the repo.


## head

```sh
wok head
```

Operate on subrepos heads.


### switch

```sh
wok head switch
```

Switch all subrepos heads to the current umbrella repo head.


## repo

```sh
wok repo
```

Manage subrepos configuration.


### add

```sh
wok repo add <SUBMODULE_PATH>
```

Add submodule previously configured in the repo at the path `<SUBMODULE_PATH>` to [Wokfile](./wokfile.md).


### rm

```sh
wok repo rm <SUBMODULE_PATH>
```

Remove submodule configured in the repo at the path `<SUBMODULE_PATH>` from [Wokfile](./wokfile.md).
