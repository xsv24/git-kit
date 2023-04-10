[![rust](https://img.shields.io/badge/rust-161923?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![git-logo](https://img.shields.io/badge/git-F05032?style=for-the-badge&logo=git&logoColor=white)](https://git-scm.com/)

[![crates.io](https://img.shields.io/crates/v/git-kit?label=%F0%9F%93%A6%20git-kit&style=flat-square)](https://crates.io/crates/git-kit)
[![Main branch tests](https://img.shields.io/github/actions/workflow/status/xsv24/git-kit/commit.yml?branch=main&label=%F0%9F%A7%AA%20tests&style=flat-square)](https://img.shields.io/github/actions/workflow/status/xsv24/git-kit/actions)
[![license](https://img.shields.io/github/license/xsv24/git-kit?color=blue&style=flat-square&logo=)](./LICENSE)

# 🧰 git-kit

Use this CLI to help format your git commit messages consistently with less effort via pre-provided templates! 🤩

There are two default templates provided:

1) [**Simple Commit Template**](#simple-commit-template)


2) [**Conventional Commit Template**](#conventional-commit-standard-templates)

You can also create your own Custom Templates by following the [**Custom Template Guide**](#-custom-commit-template-example). 

## Simple Commit Template
```bash
git-kit config set default
```

```text
-  ✨ feat        Adds new functionality.
-  🐛 bug         Fix that resolves an unintended issue.
-  🧪 test        Improves or adds existing tests related to the code base.
-  🧹 refactor    Improvement of code/structure without adding new functionality.
- 📖 docs         Change or update to documentation (i.e README's, code comments, etc).
-  📦 deps        Version update or migration to a new dependency.
-  ⚠️ break        Breaking change that may break a downstream application or service.
-  📋 chore       Regular code maintenance.
-  🏭 ci          Changes to CI configuration files and scripts.
```

### Example Commit format:
- `[{ticket_num}] ❓ {message}`


### Template Context:

- `ticket_num` ticket / issue number related to the branch.
- `message` commit message.

## Conventional Commit Standard Templates

```bash
git-kit config set conventional
```

```text
- ✨ feat        Adds new functionality.
- ⛑ fix         Fix that resolves an unintended issue (i.e. bug).
- 🧪 test        Improves or adds existing tests related to the code base.
- 🧹 refactor    Improvement of code/structure without adding new functionality.
- 📖 docs        Change or update to documentation (i.e README's, code comments, etc).
- 🔨 build       Changes that affect the build system or external dependencies.
- 📋 chore       Regular code maintenance.
- 🏭 ci          Changes to CI configuration files and scripts.
- 🏎 perf        Improvement of code performance (i.e. speed, memory, etc).
- 🕺 style       Formatting updates, lint fixes, etc. (i.e. missing semi colons).
```

### Commit format:
- `{type}({scope}): {message}`


### Template commit context:

- `ticket_num` ticket / issuer number related to the branch.
- `message` subject message.
- `scope` Short description of a section of the codebase the commit relates to.

## ⏳ Install Binary
<details>
  <summary>🦀 Rust's Cargo</summary>
 
  Install the latest version via [Rust](https://www.rust-lang.org/tools/install). 

  ```bash
  cargo install git-kit
  ```
</details>

<details>
  <summary>🍏 MacOS</summary>
  
  > Homebrew coming soon 🤞

  Install the latest version:

  ```bash
  curl -sS https://raw.githubusercontent.com/xsv24/git-kit/main/scripts/install.sh | sh
  ```

  Depending on your setup you may need to run the script with `sudo`.

  ```bash
  curl -sS https://raw.githubusercontent.com/xsv24/git-kit/main/scripts/install.sh | sudo sh -s - -b /usr/local/bin
  ```
</details>

<details>
  <summary>🐧 Linux</summary>
  
  > Package managers coming soon 🤞

  Install the latest version:

  ```bash
  curl -sS https://raw.githubusercontent.com/xsv24/git-kit/main/scripts/install.sh | sh
  ```
</details>

<details>
  <summary>🪟 Windows</summary>

  Coming soon 🤞
</details>

---
## 🏎️💨 Getting Started

```bash
git-kit --help
```

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

### 🎟️ Checkout command

Creates a new branch or checks out an existing branch attaching the following optional context parameters for use in future commit templates.

- `ticket_num` Issue number related to the branch.
- `link` Link to to the related issue.
- `scope` Short description of a section of the codebase the commit relates to.

When it's time to [commit](#commit-command) your changes any provided context params (i.e.`ticket_number`) will be injected into each commit message for you automatically! 😄 It does this by a simple template string injection.

Examples:
```bash
git-kit checkout my-branch --ticket TICKET-123
git-kit checkout my-branch \
  -t TICKET-123 \
  --scope parser \
  --link "http://ticket-manager/TICKET-123"
```

Most likely your ticket / issue will only have one branch associated to it. In this case you can use the following shorthand 👌

```bash
git-kit checkout TICKET-123
```

---
### 🔗 Context command

Create or update context params linked to the current checked out branch.

- `ticket_num` Issue number related to the branch.
- `link` Link to to the related issue.
- `scope` Short description of a section of the codebase the commit relates to.

This is handy if you forgot to add context via the `git-kit` [checkout command](#-checkout-command) or if you want to update the context for future commits.

When it's time to [commit](#commit-command) your changes any provided context params (i.e.`ticket_number`) will be injected into each commit message for you automatically! 😄 It does this by a simple template string injection.


```bash
git-kit context \
  --ticket TICKET-123 \
  --scope parser \
  --link "http://ticket-manager/TICKET-123"
```
---
### 🚀 Commit command

Commits any staged changes and builds an editable commit message by injecting any context set parameters from the [checkout](#-checkout-command) or [context](#-context-command) commands into a chosen [template](./templates/default.yml) (i.e. `bug`).

```bash
git-kit commit bug
```
> Example template:
> 
> `[{ticket_num}] 🐛 {message}` → `[TICKET-123] 🐛 Fix`
---
### ☑ Templates command

Lists currently available commit templates. To add your own, refer to the [Custom Commit Template guide](#-custom-commit-template-example).

```bash
git-kit templates

- bug | Fix that resolves an unintended issue
- ...
```
---
## ⚙️ Configuration

The [default](./templates/default.yml) template will be set as active initially but you can switch between the [provided configurations](./templates) and any added custom templates via the `config set` command.

```bash
git-kit config set
```
> ℹ️ It's not recommend to alter the default template files as they  could be replaced / updated on new releases.
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
      description: My custom commit template 🎸
      content: |
        {ticket_num} 🤘 {message}
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
  ➜ default
    conventional
    custom
```
To ensure your template has been loaded simply run the command below 👇 to see a list of the currently configured templates.

```bash
git-kit templates

- custom | My custom commit template 🎸
- ...
```

Then when your ready to commit your changes use your custom template and your done!  🪂

```bash
git-kit commit custom \
 --ticket TICKET-123 \
 --message "Dang!"
```
> [TICKET-123] 🤘 Dang!
