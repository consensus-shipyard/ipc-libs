# Functions to help set up the infrastructure needed to run end-to-end tests.
PROJECT_ROOT     := ../..
IPC_SRC					 := $(shell find $(PROJECT_ROOT) -type f \( -name "*.rs" \) | grep -v target)
IPC_AGENT        := $(PROJECT_ROOT)/bin/ipc-agent
IPC_AGENT_CONFIG := ./.ipc/agents/0/config.toml

.PHONY: clean
clean:
	rm -rf .ipc .make


# Build the IPC agent binary.
$(IPC_AGENT): $(IPC_SRC)
	$(MAKE) -C $(PROJECT_ROOT) build


# Initialize the agent config.
$(IPC_AGENT_CONFIG):
	mkdir -p $(dir $@)
	$(IPC_AGENT) --config-path $@ config init


# Build the IPC agent docker image.
.make/docker/ipc-agent: $(IPC_SRC)
	cd $(PROJECT_ROOT) && docker build -t ipc-agent .
	mkdir -p $(dir $@) && touch $@


# Make sure the IPC agent has been built.
.PHONY: ipc-agent
ipc-agent: $(IPC_AGENT) .make/docker/ipc-agent


# Make sure the eudico docker image is built.
.PHONY: eudico
eudico:
	@if [ -z "$(shell docker images | grep eudico)" ]; then \
		$(MAKE) -C $(PROJECT_ROOT) install-infra; \
	fi


# Check that jq is installed.
.PHONY: jq
jq:
	@if [ -z "$(shell which jq)" ]; then \
		echo "Please install jq. See https://stedolan.github.io/jq/"; \
		exit 1; \
	fi
