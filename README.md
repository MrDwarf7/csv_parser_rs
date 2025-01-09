# README

## Overview

This program provides a CLI tool to parse CSV files, filtering out columns & rows based on criteria specified in a configuration file (`config.json`).
Users can optionally override settings using command-line arguments, ensuring flexibility and adaptability to various use cases.

## Configuration

By default, the program will search for a config directory & config file in the following locations:
`$ROOT/config` & `$ROOT/config/config.json` respectively. (Where $ROOT is the directory the binary resides in).

If the config folder or the config file is not found, the program will generate dummy config file with similar struct and values to the below snippet.

The configuration file (`config.json`) should be formatted as follows:

```json
{
  "source": "\\windows\\path\\to\\source.csv",
  "output_type": "csv",
  "output_path": "linux_style/path/to/output.csv",
  "has_headers": true,
  "fields": [
    "Field1",
    "Field2",
    "Field3"
  ],
  "unique_fields": [
    "unique_fields_to_include"
  ],
  "include_cols_with": {
    "Field1": [
      "FilterCriteria1",
      "FilterCriteria2"
    ],
    "Field2": [
      "FilterCriteria3",
      "FilterCriteria4"
    ]
  }
}
```

- Note: The code handles both Windows and Linux-style paths.
  This does not mean the Filesystem you're interacting with via the tool will necessarily play nice.

### Fields:

- `source`: Path to the input CSV file.
- `output_type`: Desired output format (e.g., `csv`).
- `output_path`: Path for the output CSV file.
- `has_headers`: Boolean value indicating whether the CSV file has headers.
- `fields`: An array of fields to always include in the output.
- `unique_fields`: An array of fields to include in the output only if they are unique. (Optional - Leave list blank if not needed)
- `include_cols_with`: A dictionary defining filtering criteria the keys are the columns, and the list of values are values that should be included in the output.

## Command Line Interface

All commands are also available by running

```powershell
.\csv_parser_rs --help
# or
.\csv_parser_rs -h
```

You can run the parser using the following command:

```powershell
.\csv_parser_rs [source] [-c config_file] [-t output_type] [-o output_path]
```

### Arguments:

- `source`: (Optional) First argument - Path to the source CSV file; overrides the `source` in `config.json`.
- `-c, --config`: (Optional) Path to an alternative configuration file; overrides the default.
- `-t, --output_type`: (Optional) Specify the output type (`stdout`, `csv`); defaults to the value in `config.json`.
- `-o, --output_path`: (Optional) Specify the output file path; overrides the `output_path` in `config.json`.

## Output Types

The tool supports two output types:

- **stdout**: Print the results to the standard output.
- **csv**: Save the results to a specified CSV file.

## Usage Example

To run the parser with a custom configuration file (ie: One that is not in the assumed location):

```powershell
.\csv_parser_rs -c path\to\config.json
```

To override the configuration using CLI arguments:

```powershell
.\csv_parser_rs path\to\input.csv -t stdout -o path\to\output.csv
```

## Author

Blake B.

## License

This project is licensed under the MIT License.