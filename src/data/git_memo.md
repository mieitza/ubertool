# Git cheat sheet

## Setup
```
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

## Basic workflow
```
git init                       # init repo
git status                     # show working tree state
git add <path>                 # stage changes
git add -p                     # stage interactively
git commit -m "message"        # commit staged
git commit --amend             # amend last commit
git log --oneline -10          # recent commits
```

## Branches
```
git branch                     # list branches
git branch -a                  # include remotes
git switch -c <name>           # create + switch
git switch <name>              # switch
git merge <branch>             # merge into current
git rebase <branch>            # rebase current onto branch
git rebase -i HEAD~5           # interactive rebase last 5
git branch -d <name>           # delete merged branch
git branch -D <name>           # force delete
```

## Remotes
```
git remote -v                  # list
git remote add origin <url>    # add
git fetch                      # download remote refs
git pull                       # fetch + merge
git pull --rebase              # fetch + rebase
git push                       # push current branch
git push -u origin <name>      # push + set upstream
git push --force-with-lease    # safer force push
```

## Undo / inspect
```
git diff                       # unstaged vs working
git diff --cached              # staged vs HEAD
git diff <a>..<b>              # range
git log -p <path>              # changes touching a path
git blame <path>               # who wrote each line
git show <sha>                 # full commit
git restore <path>             # discard working changes
git restore --staged <path>    # unstage
git reset --soft HEAD~1        # undo commit, keep changes staged
git reset --mixed HEAD~1       # undo commit, keep changes unstaged
git reset --hard HEAD~1        # undo commit, DROP changes (!)
git revert <sha>               # new commit that undoes <sha>
```

## Stash
```
git stash                      # save uncommitted
git stash list                 # list
git stash pop                  # restore most recent
git stash apply stash@{2}      # restore specific
git stash drop stash@{2}       # forget
```

## Searching
```
git grep <pattern>             # search tracked files
git log -S<string>             # commits that add/remove <string>
git log --grep=<pattern>       # search commit messages
git bisect start ...           # binary search for a bad commit
```

## Tags
```
git tag                        # list
git tag -a v1.0.0 -m "msg"     # annotated
git push --tags                # push tags
```
