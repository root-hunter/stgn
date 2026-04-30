.PHONY: all build build-release check test test-verbose test-stgn test-wasm \
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

## Esegui solo i test del crate stgn
test-stgn:
	$(CARGO) test -p stgn

## Esegui solo i test del crate stgn-wasm
test-wasm:
	$(CARGO) test -p stgn-wasm

## Esegui un singolo test per nome (es: make test-one NAME=test_steganography)
test-one:
	$(CARGO) test -p stgn $(NAME) -- --nocapture

# ── Run ────────────────────────────────────────────────────────────────────────

## Lancia il binario CLI (es: make run ARGS="encode -i img.png -m 'hello'")
run:
	$(CARGO) run -p stgn -- $(ARGS)

## Lancia in modalità release
run-release:
	$(CARGO) run -p stgn --release -- $(ARGS)

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

## Compila stgn-wasm per WebAssembly
wasm:
	$(CARGO) build -p stgn-wasm --target $(WASM_TARGET) --release

## Genera i binding JS con wasm-pack e li mette in docs/pkg (pronti per il demo)
wasm-pack:
	rm -f docs/pkg/*
	$(WASM_PACK) build stgn-wasm --target web --out-dir ../docs/pkg

## Avvia un server HTTP locale sulla cartella docs/ (richiede python3)
serve-docs:
	python3 -m http.server 8080 --directory docs

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

cli-help:
	@echo ""
	@echo "CLI examples:"
	@echo ""
	@echo "  make run ARGS=\"encode -i input.png -o output.png --data-strings 'secret message'\""
	@echo "  make run ARGS=\"decode -i output.png --data-strings 'secret message'\""
	@echo "  make run ARGS=\"max-capacity -i input.png\""
	@echo ""