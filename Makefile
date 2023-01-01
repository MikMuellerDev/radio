DIR := ${CURDIR}

build:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target arm-unknown-linux-gnueabihf

build-docker:
	docker build . -t radio-cross:latest
