repo_root := parent_directory(justfile())

working_directory := invocation_directory()

set dotenv-load := true
set dotenv-path := "/home/byron/Documents/projects/home_network/.env"

alias b := build
alias r := run
alias c := check
alias t := test

default: 
    @just --choose

build PROJ:
    cd {{PROJ}} && cargo build

run PROJ:
    cd {{PROJ}} && cargo run

check PROJ:
    cd {{PROJ}} && cargo check

test PROJ:
    cd {{PROJ}} && cargo test
