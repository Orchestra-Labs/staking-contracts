staking-contracts-cli
=================

A command line interface to manage staking contracts for the symphony chain


[![oclif](https://img.shields.io/badge/cli-oclif-brightgreen.svg)](https://oclif.io)
[![Version](https://img.shields.io/npm/v/staking-contracts-cli.svg)](https://npmjs.org/package/staking-contracts-cli)
[![Downloads/week](https://img.shields.io/npm/dw/staking-contracts-cli.svg)](https://npmjs.org/package/staking-contracts-cli)


<!-- toc -->
* [Usage](#usage)
* [Commands](#commands)
<!-- tocstop -->
# Usage
<!-- usage -->
```sh-session
$ npm install -g staking-contracts-cli
$ symphony-staking-cli COMMAND
running command...
$ symphony-staking-cli (--version)
staking-contracts-cli/0.0.0 darwin-arm64 node-v22.11.0
$ symphony-staking-cli --help [COMMAND]
USAGE
  $ symphony-staking-cli COMMAND
...
```
<!-- usagestop -->
# Commands
<!-- commands -->
* [`symphony-staking-cli base`](#symphony-staking-cli-base)
* [`symphony-staking-cli config init MNEMONIC`](#symphony-staking-cli-config-init-mnemonic)
* [`symphony-staking-cli config show`](#symphony-staking-cli-config-show)
* [`symphony-staking-cli hello PERSON`](#symphony-staking-cli-hello-person)
* [`symphony-staking-cli hello world`](#symphony-staking-cli-hello-world)
* [`symphony-staking-cli help [COMMAND]`](#symphony-staking-cli-help-command)
* [`symphony-staking-cli plugins`](#symphony-staking-cli-plugins)
* [`symphony-staking-cli plugins add PLUGIN`](#symphony-staking-cli-plugins-add-plugin)
* [`symphony-staking-cli plugins:inspect PLUGIN...`](#symphony-staking-cli-pluginsinspect-plugin)
* [`symphony-staking-cli plugins install PLUGIN`](#symphony-staking-cli-plugins-install-plugin)
* [`symphony-staking-cli plugins link PATH`](#symphony-staking-cli-plugins-link-path)
* [`symphony-staking-cli plugins remove [PLUGIN]`](#symphony-staking-cli-plugins-remove-plugin)
* [`symphony-staking-cli plugins reset`](#symphony-staking-cli-plugins-reset)
* [`symphony-staking-cli plugins uninstall [PLUGIN]`](#symphony-staking-cli-plugins-uninstall-plugin)
* [`symphony-staking-cli plugins unlink [PLUGIN]`](#symphony-staking-cli-plugins-unlink-plugin)
* [`symphony-staking-cli plugins update`](#symphony-staking-cli-plugins-update)

## `symphony-staking-cli base`

```
USAGE
  $ symphony-staking-cli base
```

_See code: [src/commands/base.ts](https://github.com/Orchestra-Labs/staking-contracts/blob/v0.0.0/src/commands/base.ts)_

## `symphony-staking-cli config init MNEMONIC`

Initialize cli configuration

```
USAGE
  $ symphony-staking-cli config init MNEMONIC -g <value> -e <value> -p <value>

ARGUMENTS
  MNEMONIC  Mnemonic of your wallet

FLAGS
  -e, --rpcEndpoint=<value>  (required) RPC endpoint of the chain
  -g, --gasPrice=<value>     (required) Gas price string value (e.g.: 0.025ujuno)
  -p, --prefix=<value>       (required) Addresses prefix (e.g.: juno)

DESCRIPTION
  Initialize cli configuration

EXAMPLES
  $ symphony-staking-cli config init "word1 word2 ..." --gasPrice 0.1note --rpcEndpoint "localhost:443" --prefix symphony
```

_See code: [src/commands/config/init.ts](https://github.com/Orchestra-Labs/staking-contracts/blob/v0.0.0/src/commands/config/init.ts)_

## `symphony-staking-cli config show`

Show current configuration

```
USAGE
  $ symphony-staking-cli config show

DESCRIPTION
  Show current configuration
```

_See code: [src/commands/config/show.ts](https://github.com/Orchestra-Labs/staking-contracts/blob/v0.0.0/src/commands/config/show.ts)_

## `symphony-staking-cli hello PERSON`

Say hello

```
USAGE
  $ symphony-staking-cli hello PERSON -f <value>

ARGUMENTS
  PERSON  Person to say hello to

FLAGS
  -f, --from=<value>  (required) Who is saying hello

DESCRIPTION
  Say hello

EXAMPLES
  $ symphony-staking-cli hello friend --from oclif
  hello friend from oclif! (./src/commands/hello/index.ts)
```

_See code: [src/commands/hello/index.ts](https://github.com/Orchestra-Labs/staking-contracts/blob/v0.0.0/src/commands/hello/index.ts)_

## `symphony-staking-cli hello world`

Say hello world

```
USAGE
  $ symphony-staking-cli hello world

DESCRIPTION
  Say hello world

EXAMPLES
  $ symphony-staking-cli hello world
  hello world! (./src/commands/hello/world.ts)
```

_See code: [src/commands/hello/world.ts](https://github.com/Orchestra-Labs/staking-contracts/blob/v0.0.0/src/commands/hello/world.ts)_

## `symphony-staking-cli help [COMMAND]`

Display help for symphony-staking-cli.

```
USAGE
  $ symphony-staking-cli help [COMMAND...] [-n]

ARGUMENTS
  COMMAND...  Command to show help for.

FLAGS
  -n, --nested-commands  Include all nested commands in the output.

DESCRIPTION
  Display help for symphony-staking-cli.
```

_See code: [@oclif/plugin-help](https://github.com/oclif/plugin-help/blob/v6.2.25/src/commands/help.ts)_

## `symphony-staking-cli plugins`

List installed plugins.

```
USAGE
  $ symphony-staking-cli plugins [--json] [--core]

FLAGS
  --core  Show core plugins.

GLOBAL FLAGS
  --json  Format output as json.

DESCRIPTION
  List installed plugins.

EXAMPLES
  $ symphony-staking-cli plugins
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/index.ts)_

## `symphony-staking-cli plugins add PLUGIN`

Installs a plugin into symphony-staking-cli.

```
USAGE
  $ symphony-staking-cli plugins add PLUGIN... [--json] [-f] [-h] [-s | -v]

ARGUMENTS
  PLUGIN...  Plugin to install.

FLAGS
  -f, --force    Force npm to fetch remote resources even if a local copy exists on disk.
  -h, --help     Show CLI help.
  -s, --silent   Silences npm output.
  -v, --verbose  Show verbose npm output.

GLOBAL FLAGS
  --json  Format output as json.

DESCRIPTION
  Installs a plugin into symphony-staking-cli.

  Uses npm to install plugins.

  Installation of a user-installed plugin will override a core plugin.

  Use the SYMPHONY_STAKING_CLI_NPM_LOG_LEVEL environment variable to set the npm loglevel.
  Use the SYMPHONY_STAKING_CLI_NPM_REGISTRY environment variable to set the npm registry.

ALIASES
  $ symphony-staking-cli plugins add

EXAMPLES
  Install a plugin from npm registry.

    $ symphony-staking-cli plugins add myplugin

  Install a plugin from a github url.

    $ symphony-staking-cli plugins add https://github.com/someuser/someplugin

  Install a plugin from a github slug.

    $ symphony-staking-cli plugins add someuser/someplugin
```

## `symphony-staking-cli plugins:inspect PLUGIN...`

Displays installation properties of a plugin.

```
USAGE
  $ symphony-staking-cli plugins inspect PLUGIN...

ARGUMENTS
  PLUGIN...  [default: .] Plugin to inspect.

FLAGS
  -h, --help     Show CLI help.
  -v, --verbose

GLOBAL FLAGS
  --json  Format output as json.

DESCRIPTION
  Displays installation properties of a plugin.

EXAMPLES
  $ symphony-staking-cli plugins inspect myplugin
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/inspect.ts)_

## `symphony-staking-cli plugins install PLUGIN`

Installs a plugin into symphony-staking-cli.

```
USAGE
  $ symphony-staking-cli plugins install PLUGIN... [--json] [-f] [-h] [-s | -v]

ARGUMENTS
  PLUGIN...  Plugin to install.

FLAGS
  -f, --force    Force npm to fetch remote resources even if a local copy exists on disk.
  -h, --help     Show CLI help.
  -s, --silent   Silences npm output.
  -v, --verbose  Show verbose npm output.

GLOBAL FLAGS
  --json  Format output as json.

DESCRIPTION
  Installs a plugin into symphony-staking-cli.

  Uses npm to install plugins.

  Installation of a user-installed plugin will override a core plugin.

  Use the SYMPHONY_STAKING_CLI_NPM_LOG_LEVEL environment variable to set the npm loglevel.
  Use the SYMPHONY_STAKING_CLI_NPM_REGISTRY environment variable to set the npm registry.

ALIASES
  $ symphony-staking-cli plugins add

EXAMPLES
  Install a plugin from npm registry.

    $ symphony-staking-cli plugins install myplugin

  Install a plugin from a github url.

    $ symphony-staking-cli plugins install https://github.com/someuser/someplugin

  Install a plugin from a github slug.

    $ symphony-staking-cli plugins install someuser/someplugin
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/install.ts)_

## `symphony-staking-cli plugins link PATH`

Links a plugin into the CLI for development.

```
USAGE
  $ symphony-staking-cli plugins link PATH [-h] [--install] [-v]

ARGUMENTS
  PATH  [default: .] path to plugin

FLAGS
  -h, --help          Show CLI help.
  -v, --verbose
      --[no-]install  Install dependencies after linking the plugin.

DESCRIPTION
  Links a plugin into the CLI for development.

  Installation of a linked plugin will override a user-installed or core plugin.

  e.g. If you have a user-installed or core plugin that has a 'hello' command, installing a linked plugin with a 'hello'
  command will override the user-installed or core plugin implementation. This is useful for development work.


EXAMPLES
  $ symphony-staking-cli plugins link myplugin
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/link.ts)_

## `symphony-staking-cli plugins remove [PLUGIN]`

Removes a plugin from the CLI.

```
USAGE
  $ symphony-staking-cli plugins remove [PLUGIN...] [-h] [-v]

ARGUMENTS
  PLUGIN...  plugin to uninstall

FLAGS
  -h, --help     Show CLI help.
  -v, --verbose

DESCRIPTION
  Removes a plugin from the CLI.

ALIASES
  $ symphony-staking-cli plugins unlink
  $ symphony-staking-cli plugins remove

EXAMPLES
  $ symphony-staking-cli plugins remove myplugin
```

## `symphony-staking-cli plugins reset`

Remove all user-installed and linked plugins.

```
USAGE
  $ symphony-staking-cli plugins reset [--hard] [--reinstall]

FLAGS
  --hard       Delete node_modules and package manager related files in addition to uninstalling plugins.
  --reinstall  Reinstall all plugins after uninstalling.
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/reset.ts)_

## `symphony-staking-cli plugins uninstall [PLUGIN]`

Removes a plugin from the CLI.

```
USAGE
  $ symphony-staking-cli plugins uninstall [PLUGIN...] [-h] [-v]

ARGUMENTS
  PLUGIN...  plugin to uninstall

FLAGS
  -h, --help     Show CLI help.
  -v, --verbose

DESCRIPTION
  Removes a plugin from the CLI.

ALIASES
  $ symphony-staking-cli plugins unlink
  $ symphony-staking-cli plugins remove

EXAMPLES
  $ symphony-staking-cli plugins uninstall myplugin
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/uninstall.ts)_

## `symphony-staking-cli plugins unlink [PLUGIN]`

Removes a plugin from the CLI.

```
USAGE
  $ symphony-staking-cli plugins unlink [PLUGIN...] [-h] [-v]

ARGUMENTS
  PLUGIN...  plugin to uninstall

FLAGS
  -h, --help     Show CLI help.
  -v, --verbose

DESCRIPTION
  Removes a plugin from the CLI.

ALIASES
  $ symphony-staking-cli plugins unlink
  $ symphony-staking-cli plugins remove

EXAMPLES
  $ symphony-staking-cli plugins unlink myplugin
```

## `symphony-staking-cli plugins update`

Update installed plugins.

```
USAGE
  $ symphony-staking-cli plugins update [-h] [-v]

FLAGS
  -h, --help     Show CLI help.
  -v, --verbose

DESCRIPTION
  Update installed plugins.
```

_See code: [@oclif/plugin-plugins](https://github.com/oclif/plugin-plugins/blob/v5.4.32/src/commands/plugins/update.ts)_
<!-- commandsstop -->
