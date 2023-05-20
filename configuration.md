# Configuration Specification

## Root Fields

These specify general information about how the file should be processed.

### On Title - `on-title`

Optional.

Specifies what to do when a title row is found.

The valid options are `combine`, which ignores headers and processes the whole file as one, `once`, which returns an error if more than one header row is found, and `split`, which splits the file at each title row and processes the resulting sections as individual files.

If `combine` or `once` are selected, the macro will return a tuple containing the result of processing. If `split` is selected, a Vec of tuples will be returned, with one tuple per section of the file.

If no value is specified, the default is `once`.

### Processes - `processes`

Required.

Describes the processes to be executed.

A specification of the contents of each process can be found under [Processes](#processes).

The processes described here will be executed sequentially, with each process acting on the result of the previous. Every process returns a tuple containing the result from each column. Each file processed returns a tuple of these tuples.

Must be a list of maps.

## Processes

These entries each specify one process. A process is an operation on a file, and returns the processed data.

### Name - `name`

Required.

The name used to identify each process. In the future, this will be included in errors.

Must be a unique string.

### Columns - `columns`

Required.

Describes the operations on the columns in a file.

A specification of the contents of each column can be found under [Columns](#columns).

The entries correspond to the columns in the input file if they are in the first process, or columns from the previous process otherwise. The titles of the columns in the first process must match the titles of the columns in the input file.

## Columns

These entries specify an operation to be applied to each entry in a column. This operation accepts a value and returns either a validated value or an error.

### Title - `title`

Required.

The title of a column.

The titles of the columns in the first process must correspond to the titles of the columns in the input file.

Must be a unique string.

### Column Type - `column-type`

Required.

The data type of the corresponding column in the input file if this column is in the first process, or in the previous process otherwise.

Must be one of `boolean`, `integer`, `real`, `string`.

### Output Type - `output-type`

Optional.

The data type returned from this column. Likely to be the same as `column-type`, unless the `output` key is specified.

Must be one of `boolean`, `integer`, `real`, `string`.

Defaults to the value of `column-type`.

### Null Surrogate - `null-surrogate`

Optional.

A value to be treated as an empty entry if found.

The data type of this value must be `column-type`.

### Valid Values - `valid-values`

Optional.

A set of values to accept. All other values will be considered invalid.

Must be a list of `column-type`

### On Invalid - `on-invalid`

Optional.

What to do when an invalid value is found.

The valid options are:
- `abort`, which halts execution and returns an error if an invalid value is found.
- `average`, which averages the last valid value before a series of invalid values, and the first valid value after that series. This option requires that the key `valid-streak` be specified, which determines the number of consecutive valid values that must be found to end a series of invalid values.
- `delete`, which deletes the row if an invalid value is found.
- `previous`, which uses the previous value, or the value of `invalid-sentinel` if this is the first value. This option requires that the key `invalid-sentinel` be specified.
- `sentinel`, which uses the value of `invalid-sentinel`. This option requires that the key `invalid-sentinel` be specified.

If no value is specified, the default is `abort`.

### On Null - `on-null`

Optional.

What to do when a null entry is found.

This will also be used if the value of `null-surrogate` is found.

The valid options are:
- `abort`, which halts execution and returns an error if a null entry is found.
- `average`, which behaves the same as if an invalid value was found. This option requires that `on-invalid` is set to `average`.
- `delete`, which deletes the row if a null entry is found.
- `previous`, which uses the previous value, or the value of `null-sentinel` if this is the first value. This option requires that the key `null-sentinel` be specified.
- `sentinel`, which uses the value of `null-sentinel`. This option requires that the key `null-sentinel` be specified.

If no value is specified, the default is `abort`.

### Max - `max`

Optional.

The maximum value to accept.

Any values found over this value will be considered invalid.

The data type of this value must be `column-type`.

### Min - `min`

Optional.

The minimum value to accept.

Any values under this value will be considered invalid.

The data type of this value must be `column-type`.

### Output - `output`

Optional.

An expression used to calculate the result.

The permitted operations are `+`, `-` (both binary and unary), `*`, `/`, `%`, and `!`.

The following functions are provided:
- `boolean`: Convert the argument to a Boolean. Numbers will be `false` if they are equal to 0, and `true` otherwise. Strings will be `false` if they are empty, and `true` otherwise.
- `integer`: Convert the argument to an integer. Booleans will be 1 if they are `true` and 0 if they are `false`. Floats will be rounded down to the highest representable integer lower than them. Strings will be parsed into an integer, and return an error if the parsing fails.
- `real`: Convert the argument to a float. Booleans will be 1.0 if they are `true` and 0.0 if they are `false`. Strings will be parsed into an float, and return an error if the parsing fails. Note that very large integers may lose precison when converted to floats.
- `string`: Convert the argument to a string. All values will simply be converted to a textual representation.

The `value` identifier refers to the current value in this column. To refer to the current value in another column, prefix that column's name with `value_`. For example, if you wanted to refer to the current value in the 'pulse' column, you would use the identifier `value_pulse`. Note that this refers to the raw (unprocessed) value in that column. Note that if the entry is null, this access will panic. This behaviour may change in the future.

Note that this operation is not applied on an invalid value or null entry.

If no expression is specified, the default is the current value in this column.

### Ignore - `ignore`

Optional.

Whether to ignore this column and exclude it from the output of this process.

If this is set to `true`, all other settings for the column will be disregarded.

If no value is specified, the default is `false`.
