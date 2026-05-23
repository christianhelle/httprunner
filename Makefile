CARGO := cargo
WORKSPACE := --workspace

.PHONY: all build build-debug build-release test clippy publish-dry-run clean

all: build-debug

build-debug:
	$(CARGO) build $(WORKSPACE)

build-release:
	$(CARGO) build $(WORKSPACE) --release

build: build-debug build-release

test:
	$(CARGO) test $(WORKSPACE)

clippy:
	$(CARGO) clippy $(WORKSPACE) -- -D warnings

publish-dry-run:
	@for crate in $(shell $(CARGO) metadata --format-version=1 --no-deps | python3 -c "import sys,json; data=json.load(sys.stdin); [print(p['manifest_path'].rsplit('/',1)[0]) for p in data['packages']]" 2>/dev/null || find src -name Cargo.toml | xargs dirname); do \
		echo "=== Dry-run publish $$crate ==="; \
		$(CARGO) publish --dry-run --manifest-path "$$crate/Cargo.toml" || true; \
	done

clean:
	$(CARGO) clean
