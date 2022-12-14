[![rust](https://img.shields.io/badge/rust-161923?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![git-logo](https://img.shields.io/badge/git-F05032?style=for-the-badge&logo=git&logoColor=white)](https://git-scm.com/)

[![crates.io](https://img.shields.io/crates/v/git-kit?label=%F0%9F%93%A6%20git-kit&style=flat-square)](https://crates.io/crates/git-kit)
[![Main branch tests](https://img.shields.io/github/actions/workflow/status/xsv24/git-kit/commit.yml?branch=main&label=%F0%9F%A7%AA%20tests&style=flat-square)](https://img.shields.io/github/actions/workflow/status/xsv24/git-kit/actions)
[![license](https://img.shields.io/github/license/xsv24/git-kit?color=blue&style=flat-square&logo=)](./LICENSE)

# ๐งฐ git-kit

Use this CLI to help format your git commit messages consistently with less effort via pre-provided templates! ๐คฉ

There are two default templates provided:

1) [**Simple Commit Template**](#simple-commit-template)


2) [**Conventional Commit Template**](#conventional-commit-standard-templates)

You can also create your own Custom Templates by following the [**Custom Template Guide**](#-custom-commit-template-example). 

## Simple Commit Template
```bash
git-kit config set default
```

```text
-  โจ feat        Adds new functionality.
-  ๐ bug         Fix that resolves an unintended issue.
-  ๐งช test        Improves or adds existing tests related to the code base.
-  ๐งน refactor    Improvement of code/structure without adding new functionality.
- ๐ docs         Change or update to documentation (i.e README's, code comments, etc).
-  ๐ฆ deps        Version update or migration to a new dependency.
-  โ ๏ธ break        Breaking change that may break a downstream application or service.
-  ๐ chore       Regular code maintenance.
-  ๐ญ ci          Changes to CI configuration files and scripts.
```

### Example Commit format:
- `[{ticket_num}] โ {message}`


### Template Context:

- `ticket_num` ticket / issue number related to the branch.
- `message` commit message.

## Conventional Commit Standard Templates

```bash
git-kit config set conventional
```

```text
- โจ feat        Adds new functionality.
- โ fix         Fix that resolves an unintended issue (i.e. bug).
- ๐งช test        Improves or adds existing tests related to the code base.
- ๐งน refactor    Improvement of code/structure without adding new functionality.
- ๐ docs        Change or update to documentation (i.e README's, code comments, etc).
- ๐จ build       Changes that affect the build system or external dependencies.
- ๐ chore       Regular code maintenance.
- ๐ญ ci          Changes to CI configuration files and scripts.
- ๐ perf        Improvement of code performance (i.e. speed, memory, etc).
- ๐บ style       Formatting updates, lint fixes, etc. (i.e. missing semi colons).
```

### Commit format:
- `{type}({scope}): {message}`


### Template commit context:

- `ticket_num` ticket / issuer number related to the branch.
- `message` subject message.
- `scope` Short description of a section of the codebase the commit relates to.

## โณ Install Binary

### Rust
- Install [Rust](https://www.rust-lang.org/tools/install)

```bash
cargo install git-kit
```

```bash
git-kit --help
```

## ๐๏ธ๐จ Getting Started

```bash
# Checkout a new branch & add optional context params.
git-kit checkout fix-parser
  --ticket TICKET-123 \
  --scope parser \
  --link "http://ticket-manager/TICKET-123"

# Select a registered config containing templates to use.
git-kit config set

# View currently available templates on chosen config.
git-kit templates

# Commit some changes.
git-kit commit bug -m "Fix up parser"
git-kit commit chore
```
---

### ๐๏ธ Checkout command

Creates a new branch or checks out an existing branch attaching the following optional context parameters for use in future commit templates.

- `ticket_num` Issue number related to the branch.
- `link` Link to to the related issue.
- `scope` Short description of a section of the codebase the commit relates to.

When it's time to [commit](#commit-command) your changes any provided context params (i.e.`ticket_number`) will be injected into each commit message for you automatically! ๐ It does this by a simple template string injection.

Examples:
```bash
git-kit checkout my-branch --ticket TICKET-123
git-kit checkout my-branch \
  -t TICKET-123 \
  --scope parser \
  --link "http://ticket-manager/TICKET-123"
```

Most likely your ticket / issue will only have one branch associated to it. In this case you can use the following shorthand ๐

```bash
git-kit checkout TICKET-123
```

---
### ๐ Context command

Create or update context params linked to the current checked out branch.

- `ticket_num` Issue number related to the branch.
- `link` Link to to the related issue.
- `scope` Short description of a section of the codebase the commit relates to.

This is handy if you forgot to add context via the `git-kit` [checkout command](#-checkout-command) or if you want to update the context for future commits.

When it's time to [commit](#commit-command) your changes any provided context params (i.e.`ticket_number`) will be injected into each commit message for you automatically! ๐ It does this by a simple template string injection.


```bash
git-kit context \
  --ticket TICKET-123 \
  --scope parser \
  --link "http://ticket-manager/TICKET-123"
```
---
### ๐ Commit command

Commits any staged changes and builds an editable commit message by injecting any context set parameters from the [checkout](#-checkout-command) or [context](#-context-command) commands into a chosen [template](./templates/default.yml) (i.e. `bug`).

```bash
git-kit commit bug
```
> Example template:
> 
> `[{ticket_num}] ๐ {message}` โ `[TICKET-123] ๐ Fix`
---
### โ Templates command

Lists currently available commit templates. To add your own, refer to the [Custom Commit Template guide](#-custom-commit-template-example).

```bash
git-kit templates

- bug | Fix that resolves an unintended issue
- ...
```
---
## โ๏ธ Configuration

The [default](./templates/default.yml) template will be set as active initially but you can switch between the [provided configurations](./templates) and any added custom templates via the `config set` command.

```bash
git-kit config set
```
> โน๏ธ It's not recommend to alter the default template files as they  could be replaced / updated on new releases.
> 
> Instead, copy & paste the desired default template, save it somewhere, and add it to the CLI as shown in the [persist configuration guide](#persist-configuration).

### Custom templates
Creating your own templates can be done simply by creating your own configuration file [.git-kit.yml](./templates/default.yml).

Here's an example of a custom template called `custom`

```yaml
version: 1
commit:
  templates:
    custom:
      description: My custom commit template ๐ธ
      content: |
        {ticket_num} ๐ค {message}
```

Your custom configuration / templates can be provided to the CLI in one of the following ways:

- Provide a config file path via `--config` option.
- Create a `.git-kit.yml` config file within your git repositories root directory.
- Use a config file previously added / linked via `config add` subcommand as highlighted in the [persist configuration guide](#persist-configuration).

### Persist Configuration

Persisting / linking your own config file can be done by providing the file path to your config file and a reference name.

```bash
git-kit config add $CONFIG_NAME $CONFIG_PATH
```

You can add multiple config files and switch between them via `set` command.

```bash
git-kit config set $CONFIG_NAME
```
or 

```bash
# Select prompt for available configurations
git-kit config set 

> ? Configuration:  
  โ default
    conventional
    custom
```
To ensure your template has been loaded simply run the command below ๐ to see a list of the currently configured templates.

```bash
git-kit templates

- custom | My custom commit template ๐ธ
- ...
```

Then when your ready to commit your changes use your custom template and your done!  ๐ช

```bash
git-kit commit custom \
 --ticket TICKET-123 \
 --message "Dang!"
```
> [TICKET-123] ๐ค Dang!
