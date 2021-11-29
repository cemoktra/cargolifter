# CargoLifter S3 Storage #

This crate offers a S3 storage for [CargoLifter](https://github.com/cemoktra/cargolifter)

## Configuration ##

```json
"storage": {
    "S3": {
        "bucket": "<bucket name>",
        "credentials": {
            "access_key": "<access key>",
            "secret_key": "<secret key>",
            "secret_token": "<optional secret token>",
        }
    }
}
```