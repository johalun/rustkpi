#!/bin/bash
# This script reproduces all save-analysis data in the test_data directories.

# TODO currently this is broken. Cargo makes files of the form deps/save-analysis/foo-....json
# now and you need to manually trim the -... before copying. Could probably fix
# this script with a regex or something to handle that, but I don't have the script-fu.

# Data for rls-analysis. This is essentially a bootstrap. Be careful when using
# this data because the source is not pinned, therefore the data will change
# regualarly. It should basically just be used as a 'big'-ish set of real-world
# data for smoke testing.

cd ..
RUSTFLAGS=-Zsave-analysis cargo build
rm test_data/rls-analysis/*
cp target/debug/deps/save-analysis/*.json test_data/rls-analysis
cd test_data

# Hello world test case
cd hello
RUSTFLAGS=-Zsave-analysis cargo build
cp target/debug/deps/save-analysis/hello-*.json save-analysis
#RUSTFLAGS=-Zsave-analysis-api cargo build
#cp target/debug/save-analysis/hello.json save-analysis-api
cd ..

# Types
cd types
RUSTFLAGS=-Zsave-analysis cargo build
cp target/debug/deps/save-analysis/types-*.json save-analysis
cd ..

# Expressions
cd exprs
RUSTFLAGS=-Zsave-analysis cargo build
cp target/debug/deps/save-analysis/exprs-*.json save-analysis
cd ..
