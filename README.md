# Open Fair DB CLI

## Installation

1. Open the command line interface on your system (Linux people are used to it, but even on windows you have it. German: Eingabeaufforderung)
2. Copy the following line to you command line "Eingabeaufforderung"
```sh
cargo install --git https://github.com/kartevonmorgen/ofdb-cli
```
3. It need quite a while of loading. Then something is installed here: C:\Users\USERNAME\.cargo\bin\ofdb.exe
4. To start the Command-line, type in your Eingabeaufforderung: ```ofdb``` Then it starts

The Command-line-interface (CLI) works like this:
1. It first tries to read all data and find geocoordinates for every entry.
2. Then the duplicate-Checking is automatically starting. 
-  2.1 If there are no duplicates, then it gets automatically imported.
-  2.2 If there are possible duplicates, the entries will not be imported, but in the report.json are given all IDs of duplicates, which you can than check manually.
3.  Last step is the final import of all wrongly indentified duplicates and the merging of real duplicates. (This last import does not use duplicate checking API)
