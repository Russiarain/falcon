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