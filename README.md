# Mirror Uploader

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
curseforge = "1234" # CurseForge project ID (optional)
game_versions = ["1.19.4"] # Minecraft versions (required if not defined for individual projects)
file_regex = "^.+$" # Regex string to filter uploaded GitHub assets (optional)
release_level = "release" # "release", "beta" or "alpha" (optional)

[modrinth] # top-level Modrinth settings (optional)
project_id = "xyzw"

[[modrinth.dependencies]] # a top-level Modrinth dependency
project_id = "abcd1"

[[modrinth.dependencies]] # another top-level Modrinth dependency
project_id = "abcd2"
dependency_type = "optional" # or required, embedded or incompatible (required is the default)

[[projects]] # optional
loaders = ["fabric", "forge", "quilt"] # List of mod loaders (required if not defined at top level)
curseforge = "1234" # CurseForge project ID (optional)
game_versions = ["1.19.4"] # Minecraft versions (required if not defined at top level)
file_regex = "^.+$" # Regex string to filter uploaded GitHub assets (optional)

[projects.modrinth] # project-level Modrinth settings (this table overrides the top-level settings if present)
project_id = "wzyx"
```

## Secrets

Secrets are read from `mirror_upload.secrets.toml`, another TOML file specified with the `-s` option,
or the `GITHUB_TOKEN` and `CURSEFORGE_TOKEN` environment variables with the `--env-secrets` flag.

Secrets file format:
```toml
github_token = "abcd"
curseforge_token = "1234"
```
