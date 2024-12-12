# Metabuild - build automation system

## Overview

Metabuild is a build automation system that combines a script interpreter (based
on the koto language) with a dependency manager.

## Usage

### Configuration

Metabuild does not need to be configured if you just want to run simple scripts.
However, if you want to use the dependency management functionality, you must at
least specify an index URL. If you have dependencies stored in Artifactory you
will also need to setup access tokens.

Index URL
```shell
mb config set index "https://gitlab.company.com/user/index.git"
```

Artifactory tokens
```shell
mb config set-token --url https://artifactory.company.com/artifactory ABC12DEF34
```

You can show the current configuration by running
```shell
mb config show
```

All available settings can be listed by running
```shell
mb config list
```

It is also possible to remove existing configuration items via
```shell
mb config remove index
mb config remove-token --url https://artifactory.company.com/artifactory
```

By default configuration is stored in the file `~/.mb/config.toml` resp.
`%USERPROFILE%\.mb\config.toml`. However, it is possible to set per-project
configuration overrides by using the `--local` parameter in the above commands
which will store the config to `.mb/config.toml` in the current directory. This
can be useful when projects use a custom index for example.

### Self-update and version pinning

There is self-update functionality built-in which means that Metabuild can
update itself to the newest available release. This is done by running

```shell
mb update
```

which will install the latest update.

If you want to pin a project to a certain Metabuild version (which is
recommended for reproducability reasons) you can do so by creating a file
`.mb-version` in the project root folder. The file should contain the requested
version number in the format

```
0.9.0
```

When running Metabuild from that directory it will automatically download the
matching version.

### Running scripts

To execute a metabuild script you just call

```shell
mb run [filename]
```

If you don't specify a filename, metabuild will attempt to load and run
`main.koto` from the current directory.

Here is an example for a very simple metabuild script:

```
print("Hello world!")
```

Please refer to the scripting section for more details.

### Scripting

The koto language reference can be found here:

https://koto.dev/docs/0.14/language/

Metabuild supports all core and extra libraries and the following extensions:

- `arch` (archive handling)
- `cmd` (subprocess and shell command execution)
- `git` (git repository interaction)
- `http` (http client functionality)
- `io_ext` (IO extensions like file globbing, copying, tempfiles, etc)
- `net` (file download and upload)
- `sys` (OS detection, command line argument and environment interaction)
- `utils` (colored text printing, base64 encode/decode, string templating)

Additional functionality will be made available as metabuild script packages
which can be installed from the mb-center index by adding them as dependencies
to your project manifest.

Documentation for the extra libraries will follow soon.

### Dependency management

Metabuild has built-in dependency management support which uses decentralized
infrastructure, i.e. there is no central package repository. Instead it uses an
index which maps dependency names to storage locations. Storage locations can
either be Git repositories or generic Artifactory repositories.

Every Git repository can be a valid package storage location if it has semver-
compatible tags. The dependency manager will scan the repository for available
tags so it is not necessary to publish a new package release to a remote, it is
sufficient to create a new version tag.

The index is a Git repository itself which contains an `index.json` file.
Management of the index contents can be done via the Metabuild CLI.

#### Creating and managing an index

In order to create your own index you must initialize a new Git repository and
put an empty `index.json` file on the `main` branch:

```json
{}
```

The repository must be accessible from where you want to resolve dependencies
with Metabuild. Configure the URL to your index using

```shell
mb config set index <url>
```

Note that you must use SSH. Currently there is no support for HTTPS in Metabuild.

Then you can add packages to your index via

```shell
mb index add-git <name> <url>
```

where `url` points to the Git repository.

Add as many packages as you want and then run

```shell
mb index push
```

to make Metabuild push the changes to your index repository. After that you can
start resolving dependencies with Metabuild.

#### Creating packages

A Metabuild package can be any Git repository (or archive stored in Artifactory)
which contains Metabuild scripts and/or other data. Packages can have transitive
dependencies which are described in a package manifest file `manifest.toml`
which must be located at the root of the repository. There you can define
dependencies in the format


```toml
[dependencies]
core = "^1"
artifactory = ">=2.1"
tasks = "1.7.5"
```

If you want to make a release available for resolving in Metabuild all you have
to do is tag your repository with a semver compatible version name in the format
`x.y.z`.

#### Resolving dependencies

Consumer projects can have Metabuild resolve their dependencies, including all
transitive dependencies. To do so the consumer must itself have a `manifest.toml`
in its root directory. Then all you have to do is run

```shell
mb install
```

to make Metabuild resolve the graph and install all dependencies.

Dependencies are installed in the directory `.mb/deps` and are automatically
made available as module directories for the koto prelude so that the koto files
can be imported in your scripts.