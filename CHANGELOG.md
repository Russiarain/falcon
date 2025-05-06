## v0.4.0
 
New Feature
 
- You can now select all columns in a CSV file by leaving the `[[selected]]` table blank in the configuration file. This means all columns will be included without explicitly listing them.

E.g. given the following CSV file

| a | b | c | d |
|---|---|---|---|
| 0 | 1 | 2 | 3 |
| 1 | 2 | 3 | 4 |
| 2 | 3 | 4 | 5 |
| ... | ... | ... | ... |
| 99 | 100 | 101 | 102 |

If you want to select all columns from line 42 till the end, in previous versions, the configuration file would look like this:

```toml
line_start = 42

[[selected]]
name = "a"

[[selected]]
name = "b"

[[selected]]
name = "c"

[[selected]]
name = "d"
```

Starting with this version, the configuration file can be simplified as follows:

```toml
line_start = 42
```

## v0.3.2

Enhancement

- Empty cells in selected columns won't cause an error now

### Note:

If `replacement` or `transform` is configured for the selected column, the first cell of the column cannot be empty.

## v0.3.1

Bugfix

- Transform function now works independently of fraction_digits configuration

## v0.3.0
 
New Feature
 
- A transform function can be applied to numerical cells by setting `transform` in `selected`
 
Breaking Changes
 
- Global `replacement` is removed

## v0.2.0

Enhancement

- Replacements in `[selected]` can override global replacements now
- Improve performance of replacing/formatting value 
- Improve performance of iterating over lines
- Improve performance when `line_end` is configured 
- Smaller release file size (72% of previous version)

Bugfix

- Integers won't be formatted with `fraction_digits` now

## v0.1.0

Initial release