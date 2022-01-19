# Open Fair DB CLI

## Installation

```sh
cargo install --locked --git https://github.com/kartevonmorgen/ofdb-cli
```

## Usage

### CSV import

Make sure the CSV file has all required fields (an example can be found in `tests/import-example.csv`).

```sh
ofdb --api-url https://dev.api.ofdb.io import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json entries.csv
```

NOTE: replace the OpenCage API key with a valid one.
