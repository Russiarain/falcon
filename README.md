# falcon

CSV toolkit written in Rust

Features

- Columns selecting, rearranging, renaming
- Rows selecting
- Cell content replacing
- Number rounding

## Usage

```
falcon [input].csv
    process [input].csv using config from env, create [input]_1.csv
falcon [input].csv [output].csv
    process [input].csv using config from env, create [output].csv
falcon [input].csv [output].csv [conf.toml]
    process [input].csv using config from [conf.toml], create [output].csv
```

## Configuration

Please refer to [conf.toml](conf.toml)