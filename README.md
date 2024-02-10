# Open Fair DB CLI

## Installation
Open the command line interface (cli) on your system (Linux/ Windows `cmd` German: `Eingabeaufforderung`) and copy the following code-lines to you command line:

1. Make sure [Rust](https://rust-lang.org) is installed on your system.
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    and update it regularly:
   ```
   rustup update
   ```
   
3. Install this ofdb-cli for Kartevonmorgen:
   ```sh
   cargo install --locked --git https://github.com/kartevonmorgen/ofdb-cli
   ```
4. It might take a while but then it is usually installed in
  - `~/.cargo/bin/ofdb` on Linux or
  - `C:\Users\USERNAME\.cargo\bin\ofdb.exe` on Windows.

#### Update client

```sh
cargo install --locked --force --git https://github.com/kartevonmorgen/ofdb-cli
```

## Usage

### CSV Import

Make sure the CSV file has all required fields (example: [`tests/import-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/import-example.csv)). Don't give an ID, created_by, date or Version-Number. But dont forget the Licens `CC0-1.0`.

Navigate to the folder with your import.csv, i.e.: `cd C:\Users\XYZ\Project XYZ\B. Import`

```sh
ofdb --api-url https://dev.ofdb.io/v0/ import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json "import.csv"
```

NOTE: 
- replace the [OpenCage API key](https://opencagedata.com/api#quickstart) with a valid one
- Checkout the [current API versions](https://github.com/kartevonmorgen/openfairdb/blob/main/doc/src/api_usage.md#endpoints)
- Use the `--help` -Command in the cli to get the possible operation for each function. I.e.: `ofdb import --help`
- If you need additional debug-info use `ofdb RUST_LOG=debug cargo run -- --api-url https://api.ofdb.io/v0 update --patch --report-file update-patch-02-10 update-patch.csv


##### How it works:
1. It first tries to read all data in the csv and finds geocoordinates for every entry via the opencage-api.
2. Then the duplicate-Checking is automatically starting, which compares existing places 20 m around your new entry. 
-  2.1 If there are no duplicates, then it gets automatically imported.
-  2.2 If there are possible duplicates, the entries will not be imported, but in the report.json are given all IDs of duplicates
3. Update real identified duplicates manually with your new data.
3. Last step is the final import of all wrongly indentified duplicates with a forced import ignoring possible duplicates, with the following command:

#### CSV Import ignoring duplicates

If you have recieved duplicate warnings in your first import, but you are sure, that your entries are really new ones, use the additional command:

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

### Review (confirm, reject or archive entries) via csv

Make sure the CSV file has all required fields (an example can be found in [`tests/update-patch-example.csv`](https://github.com/kartevonmorgen/ofdb-cli/blob/master/tests/update-patch-example.csv).

```sh
ofdb --api-url https://dev.ofdb.io/v0/ review --email EMAIL@host.de --password PASSWORD123 "review.csv"
```
You need to have moderation rights. Register here: https://openfairdb.org/register and request to become Scout/Pilot via info@kartevonmorgen.org

