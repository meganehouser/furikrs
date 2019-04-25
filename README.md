# furikrs
furikrs is summary generator for Github activity. 

inspire of [footprint](https://github.com/laughk/footprint) and [furik](https://github.com/pepabo/furik).

## Usage
1. make ~/pit/default.yaml , ex,
```
github.com:
  access_token: YOUR_ACCESS_TOKEN
  user_name: YOUR_GITHUB_USERNAME
```


2. execute furikrs command.
```
USAGE:
    furikrs [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -p, --private    enable get data from private repositories. (default: disable)
    -V, --version    Prints version information

OPTIONS:
    -f, --from-date <from date>    set start of date formatted 'YYYY-MM-DD'. (default: today)
    -t, --to-date <to date>        set end of date formatted 'YYYY-MM-DD'. (default: today) 
```
