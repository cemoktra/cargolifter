[![Build Status](https://github.com/cemoktra/cargolifter/workflows/CI/badge.svg)](https://github.com/cemoktra/cargolifter/actions)


# CargoLifter #
This project offers an implementation of an alternate registry. Instead of having it's own auth mechanism it uses existing source control providers logins. Those are:

- Github
- Gitlab (planned)

CargoLifter uses access tokens for interacting with the backend. So each action will be impersonated. This of course requires write access and this is the way to limit.

## Cargo Login ##
### Github ###
Use a combination of u sername and personal access token like this: `<username>:<token>`

### Gitlab ###
Use your gitlab access token as cargo login token.


## Configuration ##
Configuration is done via a JSON config file.

### Service ###
```json
"web": {
    "port": 8080
}
```

### Storage ###
FileSystem storage configuration:
```json
"storage": {
    "type": {
        "FileSystem": {
            "path": "<path>"
        }
    }
}
```

S3 storage configuration (you may omit the `credentials` for S3 access as it will default to the environment variables):
```json
"storage": {
    "type": {
        "S3": {
            "bucket": "<bucket name>",
            "credentials": {
                "access_key": "<access key>",
                "secret_key": "<secret key>",
                "secret_token": "<optional secret token>",
            }
        }
    }
}
```

Files that are mirrored will automatically put in a subfolder called `mirror`.


### Backend ###
Currently the default branch must be named "main" but this will be configurable in the future.

```json
"backend": {
    "type": {
        "Github": {
            "owner": "<username>",
            "repo": "<repository>",
            "host": "<for future when hosting custom instance>",
            "cargolifter_token": "<a token to use to merge pull requests>"
        }
    }
}
```

The registry git repository must contain a prefilled `config.json` containing:
```json
{
    "dl": "http://<hostname>:<port>/api/v1/crates",
    "api": "http://<hostname>:<port>"
}
```
