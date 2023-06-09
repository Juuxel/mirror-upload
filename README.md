# Mirror Upload

A tool that mirrors GitHub releases onto Modrinth and CurseForge.

## Usage

```sh
$ mirror_upload <GITHUB_VERSION_TAG>
```

The other config details are read from files. Read below for more information.

## Config

Config is read from `mirror_upload.config.toml`, or another TOML file specified with the `-c` option.

```toml
github = "owner/repo" # GitHub repo (required)
loaders = ["fabric", "forge", "quilt"] # List of mod loaders (required if not defined for individual projects)
game_versions = ["1.19.4"] # Minecraft versions (required if not defined for individual projects)
file_regex = "^.+$" # Regex string to filter uploaded GitHub assets (optional)
release_level = "release" # "release", "beta" or "alpha" (optional)

[modrinth] # top-level Modrinth settings (optional)
project_id = "xyzw"
slug = "mymod" # the project slug, used for printing links (optional)

[[modrinth.dependencies]] # a top-level Modrinth dependency
project_id = "abcd1"

[[modrinth.dependencies]] # another top-level Modrinth dependency
project_id = "abcd2"
dependency_type = "optional" # or required, embedded or incompatible (required is the default)

[curseforge] # top-level CurseForge settings (optional)
project_id = "1234"
slug = "mymod" # the project slug, used for printing links (optional)

[[curseforge.relations]]
slug = "hello-world"
type = "required_dependency" # "required_dependency", "optional_dependency", "embedded_library",
                             # "incompatible" or "tool" (optional)

[[projects]] # optional
loaders = ["fabric", "forge", "quilt"] # List of mod loaders (required if not defined at top level)
game_versions = ["1.19.4"] # Minecraft versions (required if not defined at top level)
file_regex = "^.+$" # Regex string to filter uploaded GitHub assets (optional)

[projects.modrinth] # project-level Modrinth settings (this table overrides the top-level settings if present)
project_id = "wzyx"
version_number = "$tag" # a version number template that supports variable $tag = git version tag (optional)

[projects.curseforge] # project-level CurseForge settings (this table overrides the top-level settings if present)
project_id = "4321"
```

### Templates

Templates use a simple [Groovy-like](http://groovy-lang.org/templating.html) format:
- `$xyz` is a template reference to variable `xyz`
- `${ xyz }` is also a template reference to variable `xyz`
- `\\` is an escaped backslash
- `\$` is an escaped dollar sign

Note that unlike Groovy, Mirror Upload's template engine doesn't support
executing any kind of code, nor the JSP-style `<% %>` syntax.

## Secrets

Secrets are read from
- `mirror_upload.secrets.toml` if it exists
- another TOML file specified with the `-s` option
- the `GITHUB_TOKEN` and `CURSEFORGE_TOKEN` environment variables with the `--env-secrets` flag or
  if `mirror_upload.secrets.toml` does not exist

Secrets file format:
```toml
github_token = "abcd"
curseforge_token = "1234"
```
