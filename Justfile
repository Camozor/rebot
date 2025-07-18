run:
		cargo run

build:
		cargo build --release

format:
		cargo fmt
		yamlfmt .

docker_build version:
		echo "Building image {{version}}"
		docker build -t camzor/rebot:{{version}} .

docker_push version:
		echo "Pushing image {{version}}"
		docker push camzor/rebot:{{version}}
