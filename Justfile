run:
		cargo run

build:
		cargo build --release

format:
		cargo fmt
		yamlfmt .

docker_build:
		docker build -t camzor/rebot .

docker_push:
		docker push camzor/rebot
