[![Build Status](https://github.com/cemoktra/cargolifter/workflows/CI/badge.svg)](https://github.com/cemoktra/cargolifter/actions)


# CargoLifter #
This project offers an implementation of an alternate registry and/or crates.io mirror.

## Configuration ##
Configuration is done via a JSON config file.

### Service ###
```json
"service": {
    "port": 8080
}
```

### Storage ###
```json
"storage": {
    "type": "FileSystem",
    "path": "<path>"
}
```

Files that are mirrored will automatically put in a subfolder called `mirror`.


### Mirror ###
```json
"mirror": {
    "remote_url": "<url>",
    "clone_path": "<path>",
    "username": "optional username for commits (defaults to cargolifter)",
    "email": "optional email for commits (defaults to git@cargolifter.com)"
}
```

The mirror git repository must contain a prefilled `config.json` containing:
```json
{
    "dl": "http://<hostname>:<port>/api/v1/mirror",
    "api": "http://<hostname>:<port>/mirror"
}
```

### Registry ###
```json
"registry": {
    "remote_url": "<url>",
    "clone_path": "<path>",
    "username": "optional username for commits (defaults to cargolifter)",
    "email": "optional email for commits (defaults to git@cargolifter.com)"
}
```

The registry git repository must contain a prefilled `config.json` containing:
```json
{
    "dl": "http://<hostname>:<port>/api/v1/crates",
    "api": "http://<hostname>:<port>"
}
```