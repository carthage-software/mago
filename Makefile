.PHONY: build-image test check fix build build-wasm clean list shell help doc-linter-rules update-sponsors regen-analyzer-issue-codes publish

DOCKER_IMAGE := mago-dev
DOCKER_RUN := docker run --rm -v $$(pwd):/workspace $(DOCKER_IMAGE)

build-image:
	docker build -t $(DOCKER_IMAGE) .

test:
	$(DOCKER_RUN) just test

check:
	$(DOCKER_RUN) just check

fix:
	$(DOCKER_RUN) just fix

build:
	$(DOCKER_RUN) just build

build-wasm:
	$(DOCKER_RUN) just build-wasm

clean:
	$(DOCKER_RUN) just clean

shell:
	$(DOCKER_RUN) /bin/bash

list:
	$(DOCKER_RUN) just list

doc-linter-rules:
	$(DOCKER_RUN) just doc-linter-rules

update-sponsors:
	$(DOCKER_RUN) just update-sponsors

regen-analyzer-issue-codes:
	$(DOCKER_RUN) just regen-analyzer-issue-codes

publish:
	$(DOCKER_RUN) just publish

help:
	@echo "Mago Development Commands"
	@echo ""
	@echo "  Docker Setup:"
	@echo "    make build-image               - Build Docker image"
	@echo ""
	@echo "  Core Commands:"
	@echo "    make test                      - Run tests"
	@echo "    make check                     - Run checks (format, lint, analyze, clippy)"
	@echo "    make fix                       - Fix issues automatically"
	@echo "    make build                     - Build project in release mode"
	@echo "    make build-wasm                - Build WebAssembly module"
	@echo "    make clean                     - Clean build artifacts"
	@echo ""
	@echo "  Documentation & Maintenance:"
	@echo "    make doc-linter-rules          - Generate linter rules documentation"
	@echo "    make update-sponsors           - Update sponsors in docs"
	@echo "    make regen-analyzer-issue-codes - Regenerate analyzer issue codes"
	@echo "    make publish                   - Publish all crates to crates.io"
	@echo ""
	@echo "  Utilities:"
	@echo "    make list                      - List all just commands"
	@echo "    make shell                     - Open interactive shell in container"
	@echo "    make help                      - Show this help message"
