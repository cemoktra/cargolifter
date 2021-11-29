# CargoLifter Gitlab Backend #

This crate implements the Gitlab backend for [CargoLifter](https://github.com/cemoktra/cargolifter)

## Configuration ##

```json
"backend": {
    "Gitlab": {
        "project_id": "<project id>",
        "host": "<hosting custom instance>",
        "cargolifter_token": "<a token to use to merge pull requests>",
        "default_branch": "<default to main>"
    }
}
```