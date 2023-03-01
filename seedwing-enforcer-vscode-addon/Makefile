.PHONY: all
all: fetch

.PHONY: fetch
fetch:
	mkdir -p cli
	test -f cli/senf-linux-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v0.1.0-alpha.2/senf-linux-amd64
	test -f cli/senf-macos-amd64 || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v0.1.0-alpha.2/senf-macos-amd64
	test -f cli/senf-windows-amd64.exe || curl -sSLO --output-dir cli https://github.com/seedwing-io/seedwing-enforcer/releases/download/v0.1.0-alpha.2/senf-windows-amd64.exe
