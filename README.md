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

#### Module resources

Resources located in the installed script modules (see section about dependency
management) can be accessed via the map `metabuild.modules`. That allows to load
files from the packages directly from your scripts. So, for example, if you have
a module `core` installed you can access the file `resources/data.txt` which is
distributed with that module by using

```
resource_path = metabuild.modules.core.get_resource("resources/data.txt")
f = io.open(resource_path)
data = f.read_to_string()
```

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

to make Metabuild resolve the graph and install all dependencies:

```
Updating cache...
Resolving dependencies...
Installing dependencies...
  [*] core/1.0.1 (from Git)
```

Dependencies are installed in the directory `.mb/deps` and are automatically
made available as module directories for the koto prelude so that the koto files
can be imported in your scripts.

### Examples

Here are some script examples that you can use as reference.

#### Python venv wrapper

This module defines a `venv` class which allows managing Python virtual
environments:

```python
build_binary_dir = |path|
    match os.name()
        "windows" then io.extend_path(io_ext.absolute(path), "Scripts")
        unmatched then io.extend_path(io_ext.absolute(path), "bin")

find_interpreter_global = ||
    return io_ext.which("python3") or io_ext.which("python")

find_interpreter_local = |path|
    return io_ext.which_in("python3", path) or io_ext.which_in("python", path)

export venv = |path|
    path: path
    bin_dir: build_binary_dir(path)
    interpreter_global: find_interpreter_global()
    interpreter_local: find_interpreter_local(build_binary_dir(path))
    suppress_output: false

    quiet: |value|
        self.suppress_output = value
    
    exists: ||
        io_ext.is_dir(self.path) and self.interpreter_local != null
    
    init: ||
        core.run(self.interpreter_global, ["-m", "venv", self.path])
        self.interpreter_local = find_interpreter_local(self.path)
    
    run: |command|
        c = cmd.new(command[0]).args(command[1..]).env_remove("PYTHONHOME")
        entry_separator = match os.name()
            "windows" then ";"
            _ then ":"
        path_variable = sys.env().contains_key("Path") and "Path" or "PATH"
        c.env(
            path_variable,
            self.bin_dir + entry_separator + sys.env().get(path_variable)
        )
        if self.suppress_output
            c.stdout("null")
            c.stderr("null")
        c.execute()
    
    install_requirements: |file|
        self.run([self.interpreter_local, "-m", "pip", "install", "-r", file])
    
    python: |command|
        self.run([self.interpreter_local, "-c", command])

    pip: |command|
        self.run([self.interpreter_local, "-m", "pip", command])
```

It can be used as follows:

```scala
v = venv(".venv")
if !v.exists()
    v.init()
v.install_requirements("requirements.txt")
v.pip("install conan==2.9")
v.run("conan install .")
```

#### Filesystem interaction

```python
# Create a temporary directory (will be deleted when the object is dropped)
tempdir = io_ext.temp_dir()

# Copy all files matching a glob into the tempdir
io_ext.copy_glob("**/*.xml", tempdir.path())

# Create and write a temporary file
tempfile = io_ext.temp_file()
tempfile.write_line("Hello world")

# Rewind file and read back data
tempfile.seek(0)
hello = tempfile.read_to_string()

# Find full path to the executable behind a shell command
conan = io_ext.which("conan")
```

#### Archives

```python
# Create a .tar.gz archive
targz = arch.targz("test.tar.gz")
targz.append_file(io.extend_path(tempdir.path(), "test.xml"), "test.xml")
targz.append_dir_all("/tmp", "tmp")

# Extract an archive
dest = "/tmp/my/folder"
io_ext.mkdirs(dest)
arch.extract("test.tar.gz", dest)
```

#### TOML

```python
# Load TOML data
config = toml.from_string(r'
[core]
version = "0.1.0"
')
print(config.core.version)

# Save TOML data
data =
  core:
    version: "0.1.0"
toml_string = toml.to_string(data)
```

#### YAML

```python
# Load YAML data
config = yaml.from_string(r'
core:
  version: 0.1.0
')
print(config.core.version)
```

#### JSON

```python
# Load JSON data
config = json.from_string(r'
{
  "core": {
    "version": "0.1.0"  
  }
}')
print(config.core.version)
```

#### HTTP

```python
# Send a dummy GET request and display the status code and response
client = http.client()
request = client.get("https://reqres.in/api/users?page=2")
response = request.call()
status = response.status()
payload = response.into_string()
print("Status {status}")
print("Payload {payload}")
```

#### Shell command and subprocess execution

```python
c = cmd.new("git").arg("--version")
output = c.execute()
print(output.stdout)

cargs = cmd.split("git --version")
c = cmd.new(cargs[0]).args(cargs[1..].to_list())
output = c.execute()
re = regex.new(r"git version (?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)")
captures = re.captures output.stdout
print("Git major version {captures.major.text()}")
```

#### Regex

```python
# Replace
s = "2021-03-15, 2022-04-16 and 2023-05-17"
re = regex.new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})")
print(re.replace_all(s, "$m/$d/$y"))

# Find all
s = "This is a test string for the find all function"
re = regex.new(r"f\w*")
matches = re.find_all(s)
for m in matches
  print(m.text())
```

#### String templates

```python
context =
  size: "small"
  color: "red"

template = r'
{
  "size": "{{ size }}",
  "color": "{{ color }}"
}'

s = utils.template(template, context)
print(s)
```

#### Unit testing

```python
testsuite =
  @test pow: ||
    x = 2.pow 3
    assert_eq x, 8

  @test abs: ||
    x = -1.abs()
    assert_eq x, 1

try
  test.run_tests testsuite
catch _
  print "A test failed"
```