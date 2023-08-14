#!/usr/bin/make -f
VERSION := $(shell echo $(shell git describe --tags) | sed 's/^v//')
COMMIT := $(shell git log -1 --format='%H')

CURRENT_DIR := $(shell pwd)
BASE_DIR := $(shell basename $(CURRENT_DIR))

compile:
	@echo "Compiling Fuzion Market $(COMMIT)..."	
	@docker run --rm -v "$(CURRENT_DIR)":/code \
	--mount type=volume,source="$(BASE_DIR)_cache",target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/workspace-optimizer:0.14.0

all:
	cargo schema	
	cargo fmt
	cargo test
	cargo clippy -- -D warnings	

test:
	cargo test -- --nocapture

test-e2e:
	sh ./e2e/test_e2e.sh