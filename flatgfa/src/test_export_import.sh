#!/bin/bash
cargo run -p flatgfa --bin flatgfa -- export test_file
cargo run -p flatgfa --bin flatgfa -- import test_file
rm -f test_file