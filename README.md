## About

This CLI application lists ELF files in specified directory and groups them by user


## Help

```bash
$ findexec -h
findexec 0.1.0
This application lists ELF files in specified directory and groups them by user

USAGE:
    findexec [OPTIONS] <TARGET>

ARGS:
    <TARGET>    Target directory

OPTIONS:
        --exclude <EXCLUDE>              Exclude files which contains specified string
        --exclude-user <EXCLUDE_USER>    Exclude files by username
    -h, --help                           Print help information
    -o, --output <OUTPUT>                Output type [possible values: json]
    -r, --recursively                    Recursively list directory
    -V, --version                        Print version information
```


## Example

```bash
$ findexec /bin -o json | jq
```