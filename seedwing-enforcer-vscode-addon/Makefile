.PHONY: all
ifeq ($(DOWNLOAD_BASE),)
all: fetch
else
all: copy
endif

VERSION ?= "0.1.0-alpha.2"

.PHONY: copy
copy:
	mkdir -p cli
	cp -av $(DOWNLOAD_BASE)/*/senf-* cli/

.PHONY: fetch
fetch:
	mkdir -p cli
	test -f cli/senf-linux-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-linux-amd64
	test -f cli/senf-macos-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-macos-amd64
	test -f cli/senf-windows-amd64.exe || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v$(VERSION)/senf-windows-amd64.exe
