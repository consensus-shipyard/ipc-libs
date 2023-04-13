# Functions to help set up the infrastructure needed to run end-to-end tests.

# Parameters that can be set when calling make to act like named arguments.
# e.g. `make <target> IPC_AGENT_NR=1`
IPC_AGENT_NR     := 0
IPC_PARENT_ID    := root
IPC_CHILD_ID     := subnet-0

PROJECT_ROOT     := $(PWD)/../..
IPC_SRC					 := $(shell find $(PROJECT_ROOT) -type f \( -name "*.rs" \) | grep -v target)
IPC_AGENT        := $(PROJECT_ROOT)/bin/ipc-agent

# Name of the docker-compose project and network.
IPC_PROJECT      := ipc
IPC_NETWORK      := $(IPC_PROJECT)
IPC_VERSION      := latest

# Location of artifacts derived from input parameters.
IPC_AGENT_DIR    := $(PWD)/.ipc/agents/agent-$(IPC_AGENT_NR)
IPC_AGENT_CONFIG := $(IPC_AGENT_DIR)/config.toml


.PHONY: clean
clean:
	docker network rm $(IPC_NETWORK) || exit 0
	rm -rf .ipc .make


# Create composition directory agent-$(IPC_AGENT_NR)
agent: $(IPC_AGENT_DIR) $(IPC_AGENT_CONFIG)


# Start the agent daemon.
agent/up: agent .make/docker/network .make/docker/images/ipc-agent
	cd $(IPC_AGENT_DIR) && docker-compose -p $(IPC_PROJECT) up -d


# Stop the agent daemon and remove the agent directory.
agent/down:
	if [ -d $(IPC_AGENT_DIR) ]; then \
		cd $(IPC_AGENT_DIR) && docker-compose down && rm -rf $(IPC_AGENT_DIR); \
	fi


# Build the IPC agent binary.
$(IPC_AGENT): $(IPC_SRC)
	$(MAKE) -C $(PROJECT_ROOT) build


# Initialize the agent directory.
$(IPC_AGENT_DIR):
	mkdir -p $@
	cp template/agent/compose.yaml $@/compose.yaml

	@# The config.toml will be recreated from config.toml.orig plus the active subnets.
	cp template/agent/config.toml  $@/config.toml.orig

	@# Create .env file with overrides taken from the environment.
	export IPC_VERSION=$(IPC_VERSION) ; \
	export IPC_NETWORK=$(IPC_NETWORK) ; \
	export IPC_AGENT_NR=$(IPC_AGENT_NR) ; \
	./scripts/make-env.sh template/agent/.env $@/.env IPC_


# Initialize the agent config.
$(IPC_AGENT_CONFIG): $(IPC_AGENT_DIR)
	echo "# DO NOT EDIT! Any changes will be overwritten as subnets are added. Add changes to config.toml.orig instead." > $@
	cat $(IPC_AGENT_DIR)/config.toml.orig >> $@
	# TODO: Find all subnets and append their config to the output.


# Create the docker network every container connects to.
.make/docker/network:
	docker network create $(IPC_NETWORK)
	mkdir -p $(dir $@) && touch $@


# Build the IPC agent docker image.
.make/docker/images/ipc-agent: $(IPC_SRC)
	cd $(PROJECT_ROOT) && docker build -t ipc-agent .
	mkdir -p $(dir $@) && touch $@


# Make sure the IPC agent has been built.
.PHONY: ipc-agent
ipc-agent: $(IPC_AGENT) .make/docker/images/ipc-agent


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
