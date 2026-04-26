.PHONY: all build build-release check test test-verbose test-stng test-wasm \
        run clean fmt lint doc wasm install-wasm-target help

# ── Variabili ──────────────────────────────────────────────────────────────────
CARGO        := cargo
WASM_TARGET  := wasm32-unknown-unknown
WASM_PACK    := wasm-pack

# ── Default ────────────────────────────────────────────────────────────────────
all: build

# ── Build ──────────────────────────────────────────────────────────────────────

## Compila tutto il workspace (debug)
build:
	$(CARGO) build

## Compila tutto il workspace (release)
build-release:
	$(CARGO) build --release

## Controlla la compilazione senza produrre artefatti
check:
	$(CARGO) check

# ── Test ───────────────────────────────────────────────────────────────────────

## Esegui tutti i test del workspace
test:
	$(CARGO) test

## Esegui i test con output verboso (mostra println! e log)
test-verbose:
	$(CARGO) test -- --nocapture

## Esegui solo i test del crate stng
test-stng:
	$(CARGO) test -p stng

## Esegui solo i test del crate stng-wasm
test-wasm:
	$(CARGO) test -p stng-wasm

## Esegui un singolo test per nome (es: make test-one NAME=test_steganography)
test-one:
	$(CARGO) test -p stng $(NAME) -- --nocapture

# ── Run ────────────────────────────────────────────────────────────────────────

## Lancia il binario CLI (es: make run ARGS="encode -i img.png -m 'hello'")
run:
	$(CARGO) run -p stng -- $(ARGS)

## Lancia in modalità release
run-release:
	$(CARGO) run -p stng --release -- $(ARGS)

# ── Qualità del codice ─────────────────────────────────────────────────────────

## Formatta tutto il codice con rustfmt
fmt:
	$(CARGO) fmt --all

## Controlla la formattazione senza modificare i file
fmt-check:
	$(CARGO) fmt --all -- --check

## Esegui Clippy (linter) su tutto il workspace
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# ── Documentazione ─────────────────────────────────────────────────────────────

## Genera la documentazione e aprila nel browser
doc:
	$(CARGO) doc --no-deps --open

# ── WASM ───────────────────────────────────────────────────────────────────────

## Installa il target wasm32
install-wasm-target:
	rustup target add $(WASM_TARGET)

## Compila stng-wasm per WebAssembly
wasm:
	$(CARGO) build -p stng-wasm --target $(WASM_TARGET) --release

## Genera i binding JS con wasm-pack
wasm-pack:
	$(WASM_PACK) build stng-wasm --target web

# ── Pulizia ────────────────────────────────────────────────────────────────────

## Rimuovi gli artefatti di build
clean:
	$(CARGO) clean

# ── Help ───────────────────────────────────────────────────────────────────────

## Mostra questo messaggio di aiuto
help:
	@echo ""
	@echo "Comandi disponibili:"
	@echo ""
	@grep -E '^##' Makefile | sed 's/^## /  /'
	@echo ""
