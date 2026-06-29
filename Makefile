PIP = .venv/bin/pip

.venv:
	python3 -m venv .venv
	$(PIP) install ipykernel

.PHONY: build
build: .venv
	wasm-pack build ./jupyter_counter_frontend --target web --out-dir pkg
	env -C ./jupyter_counter_frontend webpack
	env -C ./jupyter_counter maturin build
	$(PIP) install --force-reinstall ./target/wheels/jupyter_counter-0.1.0-cp313-cp313-manylinux_2_34_x86_64.whl
