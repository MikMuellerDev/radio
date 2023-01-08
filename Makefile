DIR := ${CURDIR}
VERSION = 0.1.0
BUILD_OUTPUT_DIR = radio-$(VERSION)

# For cross-compilation to ARM
cargo-build-arm:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target arm-unknown-linux-gnueabihf

# For cross-compilation to X86_64 Musl
cargo-build-x64:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target x86_64-unknown-linux-gnu

build-docker-cargo-arm:
	docker build . -t radio-cross:latest

# For building the UI
build-web:
	rm -rf ./radio-web/dist/
	cd ./radio-web/ && npm run build

# TODO: unify these build processes

# For building distributable archive files
build-archives: build-archive-x64 build-archive-arm

build-archive-arm: build-web cargo-build-arm
	mkdir -p ./$(BUILD_OUTPUT_DIR)/radio-web
	mkdir ./$(BUILD_OUTPUT_DIR)/images
	cp ./images/example.png ./$(BUILD_OUTPUT_DIR)/images/
	cp ./radio.service ./$(BUILD_OUTPUT_DIR)/
	cp ./install.sh ./$(BUILD_OUTPUT_DIR)/
	mkdir -p dist
	cp -r ./radio-web/dist/ ./$(BUILD_OUTPUT_DIR)/radio-web/
	cp ./target/arm-unknown-linux-gnueabihf/release/radio ./$(BUILD_OUTPUT_DIR)/
	tar -cvzf dist/radio-$(VERSION)-arm-unknown-linux-gnueabihf.tar.gz ./$(BUILD_OUTPUT_DIR)/
	rm -rf $(BUILD_OUTPUT_DIR)

build-archive-x64: build-web cargo-build-x64
	mkdir -p ./$(BUILD_OUTPUT_DIR)/radio-web
	mkdir ./$(BUILD_OUTPUT_DIR)/images
	cp ./images/example.png ./$(BUILD_OUTPUT_DIR)/images/
	cp ./radio.service ./$(BUILD_OUTPUT_DIR)/
	cp ./install.sh ./$(BUILD_OUTPUT_DIR)/
	mkdir -p dist
	cp -r ./radio-web/dist/ ./$(BUILD_OUTPUT_DIR)/radio-web/
	cp ./target/x86_64-unknown-linux-gnu/release/radio ./$(BUILD_OUTPUT_DIR)
	tar -cvzf dist/radio-$(VERSION)-x86_64-unknown-linux-gnu.tar.gz ./$(BUILD_OUTPUT_DIR)
	rm -rf $(BUILD_OUTPUT_DIR)


clean:
	rm -rf $(BUILD_OUTPUT_DIR)
	rm -rf ./radio-web/dist/
	rm -rf dist
