# `treeport`

`treeport` walks the filesystem starting from a give root directory. It
attempts to categorize each directory directory it finds. Each category has
potentially many "stat" commands that fetch stats about the directory.

## Usage

Here's a config that discovers git repos and prints out how large
they are:

`treeport.toml`:

```toml
[[categories]]
name = "git"
command = ["test", "-d", ".git"]

[[categories.stats]]
name = "size"
command = ["nu", "--commands", "du . | first | get physical"]
```

Usage is pretty simple, just give `treeport` a directory to search through. By
default, the results will be printed to stdout as an array of JSON objects.

```
$ treeport --config treeport.toml ~/src
Trawling through /home/jeremy/src... found 726 repos!
Analyzing repos...
100%|█████████████████████████████████| 726/726 [00:07<00:00, 125.31it/s]

[...]
```

## Why

I use this tool to discover dirty or unpushed git repos in my `~/src`
directory. I also sometimes run out of disk space. This lets me search for
repos that are safe to delete and sort them by size.
