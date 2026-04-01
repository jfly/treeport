# `treeport`

`treeport` ("tree report") walks the filesystem starting from a given root
directory. It attempts to categorize each directory it finds. Each category has
potentially many "stat" commands that fetch stats about the directory.

Under the hood, it uses the fastest filesystem walking technique I've
discovered from [polyglot-walks](https://github.com/jfly/polyglot-walks).

## Usage

Just give `treeport` a report spec file and a directory to search through.
By default, the results will be printed to stdout as CSV data. Here, I'm
passing it into `nushell` for pretty printing:

```
$ treeport examples/git-treeport.toml --root ~/src | nu --stdin --commands '$in | from csv'
╭─────┬──────────────────────────────────────────────────────────────┬──────────┬──────────┬──────────────╮
│   # │                                    path                      │ category │  status  │ size (bytes) │
├─────┼──────────────────────────────────────────────────────────────┼──────────┼──────────┼──────────────┤
│   0 │ /home/jeremy/src/github.com/uranusjr/packaging-the-hard-way  │ git      │ synced   │       430080 │
│   1 │ /home/jeremy/src/github.com/jfly/devshed                     │ git      │ synced   │       372736 │
│   2 │ /home/jeremy/src/github.com/smallstep/cli                    │ git      │ synced   │     14794752 │
...
│ 306 │ /home/jeremy/src/github.com/jfly/noutlive/noutlive.c         │ misc     │          │              │
...
```

## Why

I use a variant of [git-treeport.toml](examples/git-treeport.toml) to discover
dirty or unpushed git repos in my `~/src` directory. I also sometimes run out
of disk space. This lets me search for repos that are safe to delete and sort
them by size.
