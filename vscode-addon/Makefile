.PHONY: all
all: fetch

VERSION ?= 0.1.0-alpha.11

TARGETS = linux-amd64 linux-aarch64 macos-amd64 macos-aarch64 windows-amd64

.PHONY: fetch
fetch:
	mkdir -p cli
	test -f cli/senf-linux-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-linux-amd64
	test -f cli/senf-linux-aarch64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-linux-aarch64
	test -f cli/senf-macos-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-macos-amd64
	test -f cli/senf-macos-aarch64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-macos-aarch64
	test -f cli/senf-windows-amd64.exe || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-windows-amd64.exe

.PHONY: publish
publish:
	rm -Rf staging && \
	mkdir -p staging && \
	cd staging && \
	for i in $(TARGETS); do \
		curl --fail -sSLO https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-$${i}.vsix ; \
	done
	cd staging && \
	vsce publish -i $(addsuffix .vsix, $(addprefix senf-, $(TARGETS)))
