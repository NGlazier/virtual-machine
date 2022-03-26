#!/bin/bash

ERR=0
INPUTS=`ls *.o`

# Build project
echo "Building PA2:"
cargo clean &>/dev/null
if ! cargo build --release &>/dev/null; then
    cargo check --release
    echo "FAILED TO BUILD!!! ABORTING!!!"
    exit 1
fi

for f in $INPUTS;
do
    ../target/release/vm $f > "${f%.o}.student"
    if ! diff -q "${f%.o}.student" "${f%.o}.expected" &>/dev/null; then
	printf "%-10s %10s\n" $f "ERROR, outputs differ"
	ERR=1
    else
	printf "%-10s %10s\n" $f "passed"
    fi
done

exit $ERR
