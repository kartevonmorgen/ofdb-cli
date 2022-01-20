# Open Fair DB CLI

## Installation

1. Make sure [Rust](https://rust-lang.org) is installed on your system.
2. Open a terminal and run the following command
   ```sh
   cargo install --locked --git https://github.com/kartevonmorgen/ofdb-cli
   ```
3. It might take a while but then it is usually installed in
  - `~/.cargo/bin/ofdb` on Linux or
  - `C:\Users\USERNAME\.cargo\bin\ofdb.exe` on Windows.

## Update

```sh
cargo install --locked --force --git https://github.com/kartevonmorgen/ofdb-cli
```

## Usage

### CSV Import

Make sure the CSV file has all required fields (an example can be found in [`tests/import-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/import-example.csv)).

```sh
ofdb --api-url https://dev.api.ofdb.io/ import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json "entries.csv"
```

### CSV Review

Make sure the CSV file has all required fields (an example can be found in [`tests/review-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/review-example.csv)).

```sh
ofdb --api-url https://dev.ofdb.io/ review --email <EMAIL> --password <PASSWORD> "review.csv"
```

NOTE: replace the OpenCage API key with a valid one.

#### Opencage API-Key

- To make the geocoding work, you need an API-Key from https://opencagedata.com/
- Please sign-up there https://opencagedata.com/users/sign_up (SSO with Github)
  and then you find directly your API-Keys: https://opencagedata.com/dashboard#api-keys
