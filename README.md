[![rust](https://img.shields.io/badge/rust-161923?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![git-logo](https://img.shields.io/badge/git-F05032?style=for-the-badge&logo=git&logoColor=white)](https://git-scm.com/)

[![crates.io](https://img.shields.io/crates/v/git-kit?label=%F0%9F%93%A6%20git-kit&style=flat-square)](https://crates.io/crates/git-kit)
[![Main branch checks](https://img.shields.io/github/workflow/status/xsv24/git-kit/Commit%20CI?label=%F0%9F%91%8C%20checks&style=flat-square)](https://github.com/xsv24/git-kit/actions)(https://github.com/xsv24/git-kit/actions)
[![license](https://img.shields.io/github/license/xsv24/git-kit?color=blue&style=flat-square&logo=)](./LICENSE)

# 🧰 git-kit

cli to help format your git commit messages consistently with less effort via pre-provided templates 🤩

```text
-  ⚠️ break       Breaking change that could break a consuming application
-  🐛 bug         Fix that resolves an unintended issue
-  📦 deps        Dependency update or migration to a new dependency
-  📖 docs        Documentation change
-  ✨ feature     Adds new functionality
-  🧹 refactor    Improvement of code / structure without adding new functionality
-  🧪 test        Adds or improves the existing tests related to the code base
```

> - `[TICKET-123] 🐛 fix`
> - `[TICKET-123] 🧹  Clean up`

## 🥽 Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install)

## ⏳ Install Binary

```bash
cargo install git-kit
```

```bash
git-kit --help
```

## 🏎️💨 Getting Started

### 🛂 Checkout command

Creates or checks out an existing git branch and adds a ticket number as context against that branch for future commits.

So now you don't have to remember the ticket number associated to the branch! 💡.

When it's time to [commit](#commit-command) your changes the provided ticket number will be injected into each commit message </br>
thats created on the linked branch for you automatically! 😄

```bash
git-kit checkout my-branch -t TICKET-123
```
> This will create or checkout a branch named `my-branch` & link `TICKET-123` as the ticket number context to inject on any future commits on the branch named `my-branch`.

Most likely your ticket / issue will only have one branch associated to it in this case you can use the following shorthand 👌

```bash
git-kit checkout TICKET-123
```
> This will create or checkout a branch `TICKET-123` & link `TICKET-123` as the ticket number context to inject on any future commits on the branch `TICKET-123`.

### 🛂 Context command

Create or update context linked to the current checked out branch.

This is handy if you forgot to checkout by the provided `git-kit` [checkout command](#checkout-command) or if you've made a typo
in on the provided ticket number.

Again when it's time to [commit](#commit-command) your changes the provided ticket number will be injected into each commit message </br>
thats created on the linked branch for you automatically! 

```bash
git-kit context TICKET-123
```

### 🛃 Commit command

Commits your changes with a formatted message with your ticket number injected if provided from the [checkout](#checkout-command) or the [context](#context-command) command.

When committing you can specify a template to use to help describe the changes made within your commit.

```text
-  ⚠️ break       Breaking change that could break a consuming application
-  🐛 bug         Fix that resolves an unintended issue
-  📦 deps        Dependency update or migration to a new dependency
-  📖 docs        Documentation change
-  ✨ feature     Adds new functionality
-  🧹 refactor    Improvement of code / structure without adding new functionality
-  🧪 test        Adds or improves the existing tests related to the code base
```

```bash
git-kit commit bug -m "fix"
```
> This will create an editable commit with the following format and will insert branch name will be injected by default into the `bug` commit template.
>
> `[TICKET-123] 🐛 fix`


## ⚙️ Settings 

```bash
git-kit --help
```

## 🎮 Overriding

Planning on providing a way to configure your own templates at a global or repository level.
