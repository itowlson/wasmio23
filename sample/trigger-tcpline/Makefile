.PHONY: prereqs
prereqs:
	spin plugins install --url https://github.com/itowlson/spin-pluginify/releases/download/canary/pluginify.json --yes

.PHONY: build
build:
	cargo build --release

.PHONY: install
install: build
	spin pluginify
	spin plugins uninstall trigger-tcpline
	spin plugins install --file trigger-tcpline.json --yes

.PHONY: guest
guest:
	spin build -f guest/spin.toml

.PHONY: test
test: install guest
	spin up -f guest/spin.toml
