# `src-report`

Recursively search through a given directory for code repositories and produce
a report on the status of those repositories.

If you have only synced repositories, feel free to throw your computer in
the ocean!

## Usage

Usage is pretty simple, just give `src-report` a directory to search through
and a directory to put the results in. This may take a little while, but
there's a progress bar to keep you entertained.

```
$ src-report ~/src /tmp/results
Trawling through /home/jeremy/src... found 726 repos!
Analyzing repos...
100%|█████████████████████████████████| 726/726 [00:07<00:00, 125.31it/s]

# Summary
Found 611 synced repos (see /tmp/results/synced.txt)
Found 81 dirty repos (see /tmp/results/dirty.txt)
Found 43 unpushed repos (see /tmp/results/unpushed.txt)
Found 2 misc files not inside of a repo (see /tmp/results/misc.txt)
```

## Why

I do a lot of open source work, and as a result, I end up cloning a *lot* of
repositories onto my laptop. I don't like worrying about if I have anything
precious on my laptop, so run this tool periodically to make sure everything is
clean. This is especially useful if I'm about to reformat my laptop.

I also sometimes run out of disk space. I can run this tool and then delete
clean repos to free space.

## Known issues

`git` isn't the only [VCS] out there, but it's the only one this tool knows
about. PRs welcome for more VCSes!

[VCS]: https://en.wikipedia.org/wiki/List_of_version-control_software
