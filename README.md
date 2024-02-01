# Open Fair DB CLI

## Installation

1. Make sure [Rust](https://rust-lang.org) is installed on your system.
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
3. Open a terminal and run the following command
   ```sh
   cargo install --locked --git https://github.com/kartevonmorgen/ofdb-cli
   ```
3. It might take a while but then it is usually installed in
  - `~/.cargo/bin/ofdb` on Linux or
  - `C:\Users\USERNAME\.cargo\bin\ofdb.exe` on Windows.

### Update client

```sh
cargo install --locked --force --git https://github.com/kartevonmorgen/ofdb-cli
```

## Usage

### CSV Import

Make sure the CSV file has all required fields (an example can be found in [`tests/import-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/import-example.csv)).

```sh
ofdb --api-url https://dev.ofdb.io/v0/ import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json "entries.csv"
```

NOTE: replace the OpenCage API key with a valid one.

#### CSV Import ignoring duplicates

If you have recieved duplicate wanrings in your first import, but you are sure, that your entries are really new ones, use the additional command:

```sh
--ignore-duplicates
```

### Update Entries

```sh
ofdb --api-url https://dev.ofdb.io/v0/ update updates.json
```
The file to update must be an array of entries:
```json
[
  { "id": "...", "title": ".." }
]
```

#### Update (Patch) entries via csv

```sh
ofdb --api-url https://dev.ofdb.io/v0/ update --patch --report-file update-report.json update.csv
```
- you have to increase the version number manually in your csv
- leave the licencse form empty, you can't patch the license
Make sure the CSV file has all required fields (an example can be found in [`tests/update-patch.csv`]([https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/review-example.csv](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/update-patch.csv))).

#### Opencage API-Key

- To make the geocoding work, you need an API-Key from https://opencagedata.com/
- Please sign-up there https://opencagedata.com/users/sign_up (SSO with Github)
  and then you find directly your API-Keys: https://opencagedata.com/dashboard#api-keys

### CSV Review

Make sure the CSV file has all required fields (an example can be found in [`tests/review-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/review-example.csv)).

```sh
ofdb --api-url https://dev.ofdb.io/v0/ review --email EMAIL@host.de --password PASSWORD123 "review.csv"
```
You need to have moderation rights. Register here: https://openfairdb.org/register and request to become Scout/Pilot via info@kartevonmorgen.org

