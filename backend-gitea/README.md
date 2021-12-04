# CargoLifter Gitea Backend #

This crate implements the Gitea backend for [CargoLifter](https://github.com/cemoktra/cargolifter)

## Configuration ##

```json
"backend": {
    "Gitea": {
        "host": "<host name>",
        "owner": "<username>",
        "repo": "<repository>",
        "cargolifter_token": "<a token to use to merge pull requests>",
        "default_branch": "<default to main>"
    }
}
```