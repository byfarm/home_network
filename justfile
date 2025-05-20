repo_root := parent_directory(justfile())

working_directory := invocation_directory()

set dotenv-load := true
set dotenv-path := "/home/byron/Documents/projects/home_network/.env"

alias b := build
alias r := run
alias c := check
alias t := test

size PROJ:
    cd {{PROJ}} && cargo size --release

build PROJ *ARGS:
    cd {{PROJ}} && cargo build {{ARGS}}

run PROJ *ARGS:
    cd {{PROJ}} && cargo run {{ARGS}}

check PROJ *ARGS:
    cd {{PROJ}} && cargo check {{ARGS}}

test PROJ *ARGS:
    cd {{PROJ}} && cargo test {{ARGS}}
