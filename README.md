# Open Fair DB CLI

## Installation / Update

1. Open the command line interface on your system (Linux people are used to it, but even on windows you have it. German: Eingabeaufforderung)
2. Copy the following line to you command line
```sh
cargo install --git https://github.com/kartevonmorgen/ofdb-cli
```
3. It needs quite a while of loading. Then something is installed here: C:\Users\USERNAME\.cargo\bin\ofdb.exe
4. To start the Command-line, type: ```ofdb``` Then it starts


#### The Command-line-interface (CLI) works like this:
1. It first tries to read all data and find geocoordinates for every entry.
2. Then the duplicate-Checking is automatically starting. 
-  2.1 If there are no duplicates, then it gets automatically imported.
-  2.2 If there are possible duplicates, the entries will not be imported, but in the report.json are given all IDs of duplicates, which you can than check manually.
3.  Last step is the final import of all wrongly indentified duplicates and the merging of real duplicates. (This last import does not use duplicate checking API)



### 1. CSV import
1. Make sure the CSV file has all required fields: This is how the first row should look like:
```csv
title,description,lat,lng,street,zip,city,country,state,contact_name,contact_email,contact_phone,opening_hours,founded_on,tags,homepage,license,image_url,image_link_url
```
Don't give an ID, created_by, date or Version-Number. But dont forget the Licens. You are welcome to use this template: https://blog.vonmorgen.org/import-template

2. Change directory to the folder, where your Import.csv is located (with the command "CD Path-to-folder")
3. Then execute the following command (if your import-csv is called "entries.csv":
```sh
ofdb --api-url https://dev.ofdb.io/v0/ import --opencage-api-key 2049603a30ec4cb8a96c2c7fe662dc96 --report-file import-report.json import.csv
```
NOTE: replace the OpenCage API key with a valid one.

#### Opencage API-Key
To make the geocoding work, you need an API-Key from https://opencagedata.com/
Please sign-up there https://opencagedata.com/users/sign_up (SSO with Github) and then you find directly your API-Keys: https://opencagedata.com/dashboard#api-keys

### 2. Force to create new
If the duplicate-checking still returns errors, although you have checkt them and you are sure, that you are importing unique entries, than you can run this import, without the duplicate checking:

....


### 3. Update existing entries
....


### 4. Archive / reject existing entries

....
