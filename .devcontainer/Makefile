
SLOT=$(shell cat slot.txt)
USER=node

TAG=mtg-forge

# CONTAINER=docker
CONTAINER=podman

build:
	$(CONTAINER) build -t $(TAG)-$(SLOT) .

run:
	$(CONTAINER) run -it --rm -u $(USER) \
		-v `pwd`/home:/home/node \
		-v `pwd`/..:/workspace \
		$(TAG)-$(SLOT) bash

root:
	$(MAKE) USER=root run

run_old:
	$(CONTAINER) run -it --rm \
		-v `pwd`/claude-persist/.claude:/home/node/.claude \
		-v `pwd`/claude-persist/.claude.json:/home/node/.claude.json \
		$(TAG)-$(SLOT) bash
