# CargoLifter Github Backend #

This crate implements the Github backend for [CargoLifter](https://github.com/cemoktra/cargolifter)

## Configuration ##

```json
"backend": {
    "Github": {
        "owner": "<username>",
        "repo": "<repository>",
        "host": "<for future when hosting custom instance>",
        "cargolifter_token": "<a token to use to merge pull requests>",
        "default_branch": "<default to main>"
    }
}
```