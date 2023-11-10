.PHONY: all build test lint license check-fmt check-clippy diagrams install-infra clean-infra

# Make sure this tag matches the one in Cargo.toml
IPC_ACTORS_TAG				?= origin/dev
IPC_ACTORS_DIR        := $(PWD)/../ipc-solidity-actors
IPC_ACTORS_CODE       := $(shell find $(IPC_ACTORS_DIR) -type f -name "*.sol")
IPC_ACTORS_ABI        := .make/.ipc-actors-abi
IPC_ACTORS_OUT        := $(IPC_ACTORS_DIR)/out


all: test build

build:
	cargo build --release -p ipc-cli && mkdir -p bin/ && cp target/release/ipc-cli ./bin/ipc-cli

test:
	cargo test --release --workspace --exclude ipc_e2e itest

itest:
	cargo test -p itest --test checkpoint -- --nocapture

e2e:
	cargo test --release -p ipc_e2e

clean:
	cargo clean

lint: \
	license \
	check-fmt \
	check-clippy

license:
	./scripts/add_license.sh

install-infra:
	./scripts/install_infra.sh

clean-infra:
	rm -rf ./bin/ipc-infra

check-fmt:
	cargo fmt --all --check

check-clippy:
	cargo clippy --all --tests -- -D clippy::all

diagrams:
	$(MAKE) -C docs/diagrams

check-diagrams: diagrams
	if git diff --name-only docs/diagrams | grep .png; then \
		echo "There are uncommitted changes to the diagrams"; \
		exit 1; \
	fi

# Compile the ABI artifacts of the IPC Solidity actors.
ipc-actors-abi: $(IPC_ACTORS_ABI)

# Check out the IPC Solidity actors if necessary so we get the ABI artifacts, putting down a marker at the end.
$(IPC_ACTORS_ABI): $(IPC_ACTORS_CODE)
	if [ ! -d $(IPC_ACTORS_DIR) ]; then \
		mkdir -p $(IPC_ACTORS_DIR) && \
		cd $(IPC_ACTORS_DIR) && \
		git clone https://github.com/consensus-shipyard/ipc-solidity-actors.git .; \
	fi
	cd $(IPC_ACTORS_DIR) && \
	git fetch origin && \
	git checkout $(IPC_ACTORS_TAG)
	@# The ABI are already checked in; otherwise we'd have to compile with foundry
	@# make -C $(IPC_ACTORS_DIR) compile-abi
	mkdir -p $(dir $@) && touch $@
