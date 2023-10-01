# Underdose (WIP)

Dotfile overdosers' ideal companion. It's recommended to alias the binary `underdose` to `ud`. Distribution through cargo, AUR and homebrew are planned.

## WIP Warning

This artifact is still under construction. The declared features have not been fully implemented yet.

## Motivation and Philosophy

Three assertions for our potential users:

- Produce dotfiles from day to day and want to backup them up or synchronize them between machines.
- Consider a centralized git repository as a feasible way of managing dotfiles.
- Can do git, and expect the toolchain to be simple and surprise-free.

I hope `underdose` can be a powerful candidate as a tool that meets the said demands.


## User Interface

### Prerequisite

A git repository with two requirements is needed:

1. has a valid `Drugstore.toml` file;
2. has sub-directories that follows the rules listed in `Drugstore.toml`.

The git repository can be ready at any moment. The branches don't matter; the history doesn't matter; just make sure that the work tree is clean, i.e. nothing untracked, nothing staged, everything is managed by git.

`underdose` only cares about syncing the dotfiles on the machine and the local repository; the remote repository remains intact in any `underdose` operation, which means:

**Thee shall set up the git repository and pledge to maintain it by yourself.**

The bright side is you are free to use any combination of toolchains that is comfortable for you.

### Drugstore Lifespan

Prepare a local git repo. Retrive it if it's remote. Anywhere you would like is fine. From now on we call it `drugstore`.

Run `ud init` under `drugstore`. Then you'll be directed to the `Underdose.toml` setting on your machine if you have `$EDITOR` ENVVAR, effectively running `ud config`.

Don't feel lost if you forget where the repo is! `ud where` will back you up.

### `Drugstore.toml` Setup

Contains two parts.

1. `env` information to decide whether some pills need to be synced on the particular machine;
2. `pill`s for each collection of `drip`s to be managed.

```toml
# A simple pill about the nvim configuration. All machines follow the same
# configuration, and under the same path, so there is no subtlety.
[[pill]]
name = "nvim"
[[pill.drip]]
site = "~/.config/nvim"
[[pill.drip.arrow]]
site = "."
src.link = "."
```

The `drip`s can be viewed as task groups and `arrow`s can be viewed as tasks. For more examples, check `templates/Drugstore.toml`.

The configuration file for local `underdose`, named `Underdose.toml`, can be accessed through CLI (`ud config`, see below). A sample (and detailed tutorial) can be found in `template/Drugstore.toml`.

### Daily Usage

`ud sync` (poor choice of word) ensures that your machine is in sync with your *local repo*. It can be treated as an install of a drugstore to your machine or an update to the drugstore. `underdose` will try to do the following step by step, and will abort on error, which requires your manual fix before another round of `ud sync`:

1. If the drugstore doesn't have a clean work tree, abort and prompt the user to stage and commit.
2. Forall pills, `underdose` observes all drips on the machine and see whether it's not in sync, fixably not in sync, or in sync.
   1. For links, not in sync means it's not correctly pointing to the location inside drugstore.
   2. For git, not in sync means it's not the right repo; and fixably not in sync means it's the correct repo but not pulled.
   3. For collector, // Todo: ..
3. After all the preparations, `underdose` prompts you that local sync is done and you should run remote sync.

Note that these commands come at pill-level granularity, so you can operate on one specific pill:

```bash
ud sync nvim
```

After `ud sync`, you should consider a repo level sync with remote by `git pull` and `git push` to keep your machine drips up-to-date.

And that's it! Enjoy.

## Disclaimer

This software is at alpha stage and may contain unforeseen bugs. Ironically, even though it's a backup toolchain, please back things up by yourself. Luckily, `underdose` doesn't litter in your system so other backup tools can easily manage things as before.
