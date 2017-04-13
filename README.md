# How to run (for martin)
The binary should be at the root of the archive, just run it and it should
connect to 127.0.0.1:54321 automatically. For different IP/port, see
`./allie --help`.

It's statically linked so it shouldn't require anything special to be installed.

# How to compile
Just run `cargo build --release`, tested on rustc 1.16.0 (stable) and
1.18.0-nightly (c58c928e6 2017-04-11).
