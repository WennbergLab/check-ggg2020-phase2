# check-ggg2020-phase2
Utility program to check that GGG2020 Phase 2 files have all expected updates from Phase 1

## Installation
### Precompiled binary
A precompiled binary is available for 64-bit Linux OSes on the [Releases page](https://github.com/WennbergLab/check-ggg2020-phase2/releases).
To use, download and decompress the `.zip` file, and make the `check-phase2` program executable (`chmod u+x check-phase2`).

### From source
To compile this from source, you will need a version of the [Rust compiler and its Cargo package manager](https://www.rust-lang.org/). 
For Windows/Mac this is the only option at present.

Rust/Cargo v1.5.0 or newer is recommended, though older versions *may* work.
To compile, type `cargo build --release` in the top directory of this repository (the one containing the `Cargo.toml` file).
The executable will be produced in the `target/release` subdirectory.
You can leave it there, move it, or link to it as you prefer.

By default, this tries to build its own copy of the netCDF4 library to use.
If you already have the netCDF4 C library on your computer, you can use that instead by commenting out the `features = ["static"]` line at the end of `Cargo.toml`.
Note that building with `features = ["static"]` enabled currently fails for Windows & Mac virtual machines on GitHub, so you may need the netCDF library installed on your computer for this to work.

## Usage
This program requires one argument on the command line: the path to the `.private.nc` file to check:

```
check-phase2 pa20040721_20041222.private.nc
```

If successful, you will see a message similar to:

```
pa20040721_20041222.private.nc PASSES all tests - it appears to be a correct Phase 2 file
```

If not, you will see a message similar to:

```
pa20040721_20041222.private.nc FAILS at least one test - it may be a Phase 1 file or there was a problem in processing.
```

For more information on why it failed, use `-v` flags. More v's will produce more information (with `-vvvv` being the most information):

```
$ check-phase2 -v pa20040721_20041222.private.nc
* FAIL: ADCFs do not match expected values
* FAIL: AICFs do not match expected values
* FAIL: Window-to-window scale factors do not match expected values
* PASS: All windows expected to be present are
* FAIL: At least one window expected to have been removed is present
* FAIL: At least one program version does not match expected
* FAIL: 62/2556 expected InGaAs variables missing

pa20040721_20041222.private.nc FAILS at least one test - it may be a Phase 1 file or there was a problem in processing.
```

To only show failing tests, use the `-f` flag.
