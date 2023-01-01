# Underdose (WIP)

Dotfile overdosers' ideal companion. It's recommended to alias the binary `underdose` to `ud`. Distribution through cargo, AUR and homebrew are planned.

## WIP Warning

This artifact is still under construction. The declared features have not been fully implemented yet.

## Motivation and Philosophy

Three assertions for our potential users:

- Produce dotfiles from day to day and want to backup them up or synchronize them between machines.
- Consider a centralized git repository as a feasible way to manage dotfiles.
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

Prepare a local git repo before we talk. Retrive it if it's remote. Anywhere you would like is fine. From now on we call it `drugstore`.

Run `ud init <path/to/drugstore>`. Then you'll be directed to the `Underdose.toml` setting on your machine if you have `$EDITOR` ENVVAR, effectively running `ud config`.

Don't feel lost if you forget where the repo is! `ud where` will back you up.

### `Drugstore.toml` Setup

Contains two parts.

1. `env` information to decide whether some pills need to be synced on the particular machine;
2. `pill`s for each collection of drips to be managed.

```toml
# A simple pill about the nvim configuration. All machines follow the same
# configuration, and under the same path, so there is no subtlety.
[[pill]]
name = "nvim"
[[pill.drip]]
site = "~/.config/nvim"
stem = [""]
# "" stands for current dir
```

The configuration file for local `underdose`, named `Underdose.toml`, can be accessed through CLI (`ud config`, see below). A sample (and detailed tutorial) can be found in `template/Drugstore.toml`.

### Daily Usage

`ud sync` (poor choice of word) ensures that your machine is in sync with your *local repo*. It can be treated as an install of a drugstore to your machine or an update to the drugstore. `underdose` will try to do the following step by step, and will abort on error, which requires your manual fix before another round of `ud sync`:

1. If the drugstore doesn't have a clean work tree, abort and prompt the user to stage and commit.
2. Forall pills, `underdose` collects all drips on the machine and see whether it's an unsynced visit.
   1. If it's a new visit, i.e. different from the state of the last visit, machine overwrites local, and prompts for manual commit / tweak.
   2. If the visit is already synced, `ud pour` can be run automatically by `underdose`. It's safe because the visit is synced to drugstore and thus took into git's consideration, and the new change will come from remote.
3. After all the preparations, `underdose` prompts you that local sync is done and you should run remote sync.

`ud pour` works the other way around. If we consider `ud sync` to be overwriting file in drugstore to respect the drips on the machine, `ud pour` overwrites the local machine file to keep it inline with your drugstore, namely pouring the drips. Though backups are preserved in cache, it's potentially dangerous, so `ud pour -f` is always required. To undo an immediate pour one may use `ud unpour`.

Note that these commands come at pill-level granularity, so you can operate on one specific pill:

```bash
ud sync nvim

# tweaking config on machine and accidentally ...
ud pour nvim -f
# oops, what have I done?

ud unpour nvim
# safe ... for now ...
```

After `ud sync`, you should consider a repo level sync with remote by `git pull` and `git push` followed by a final `ud sync --pour` to keep your machine drips up-to-date.

And that's it! Enjoy.


## Under the Hood

The following sections demonstrate the technical details of `underdose`.

### Details of `ud sync` and State Management

`ud sync` is composed of running `ud sync <pill>` for all pill and a final check of whether the tree is clean for remote sync. Upon each `ud sync <pill>`, a temporary folder will be generated and all machine drips will be poured in. Then its sha512sum will be recorded. Only the lastest sha512sum will be recorded; a "synced" visit would mean the sha512sum hasn't changed, which makes it safe to pour the updated pill from local or remote.

### Core component explained - `PillTask` and `AtomTask`

...

### Roadmap

- [ ] Carefully design the task primitives.
- [ ] Snapshot before sync. Use a table of hashes for snapshot. Use a file-lock for parallel operations.
- [ ] Undo.
- [ ] Automatic sync. Allow scheduled run for trivial tasks, but abort when anything complicated emerges. Useful for auto git fetch and pour when your remote drugstore updates.
- [ ] Probe for changes during `ud sync` even without being asked to. It's like running `git fetch` from time to time without asking can actually save your time, but to your pills instead.
- [ ] Symlink support. Treat symlink as the file it points to and keeps track of both the pointee and pointer, namely sync the file and create the symlink automatically.
- [x] Cascaded item under dir track. Assign a dir some option and some specific files other options.
- [x] Globs in paths.
- [ ] Hooks for pre- and post- actions.


## Disclaimer

This software is at alpha stage and may contain unforeseen bugs. Ironically, even though it's a backup toolchain, please back things up by yourself. Luckily, `underdose` doesn't litter in your system so other backup tools can easily manage things as before.
