# Open Fair DB CLI

## Installation

```sh
cargo install --git https://github.com/kartevonmorgen/ofdb-cli
```

## Usage

### CSV import

Make sure the CSV file has all required fields.

This is how the first row should look like:

```csv
title,description,lat,lng,street,zip,city,country,state,contact_name,contact_email,contact_phone,opening_hours,founded_on,tags,homepage,license,image_url,image_link_url
```

```sh
ofdb --api-url https://dev.api.ofdb.io import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json entries.csv
```

NOTE: replace the OpenCage API key with a valid one.
