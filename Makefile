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

build-cargo-arm-docker:
	docker build . -t radio-cross:latest

# For building the UI
build-web:
	rm -rf ./radio-web/dist/
	cd ./radio-web/ && npm run build

# For building distributable archive files
build-archive-arm: build-web cargo-build-arm
	mkdir -p $(BUILD_OUTPUT_DIR)/radio-web
	mkdir -p dist
	cp -r ./radio-web/dist/ ./$(BUILD_OUTPUT_DIR)/radio-web/
	cp ./target/arm-unknown-linux-gnueabihf/release/radio ./$(BUILD_OUTPUT_DIR)
	tar -cvzf dist/radio-$(VERSION)-arm-unknown-linux-gnueabihf.tar.gz ./$(BUILD_OUTPUT_DIR)
	rm -rf $(BUILD_OUTPUT_DIR)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)
	rm -rf ./radio-web/dist/
	rm -rf dist
