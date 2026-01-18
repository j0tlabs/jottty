# Jottty.

Jottty is a simple and efficient note-taking application designed to help you capture your thoughts, ideas, 
and to-do lists quickly. With a clean and intuitive interface, Jottty makes it easy to organize your notes
and access them whenever you need.

![img](./img/logo.svg)

## What jot means?

To jot = to make a short, quick note so you don’t forget.

![view](./img/view.svg)

# How to install.


## Usage

```bash
> jottty add "This is a bullet note in today's journal"

> jottty list
journals/
  ├── 2026-01-10.md
  ├── 2026-01-09.md
  └── 2026-01-08.md


> jottty view 2026-01-09    # View journal of a specific date
# 2026-01-09
- This is a bullet note in past journal


> jottty view              # View today's journal
# 2026-01-10
- This is a bullet note in today's journal


> jottty add "TODO: Finish the project"

> jottty view
# 2026-01-10
- TODO: Finish the project
- This is a bullet note in today's journal


> jottty search "TODO"

> jottty tag --filter "TODO"

> jottty edit              # Open today's journal in editor
```
