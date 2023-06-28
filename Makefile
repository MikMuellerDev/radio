DIR := ${CURDIR}
VERSION = 0.2.0
BUILD_OUTPUT_DIR = radio-$(VERSION)

.PHONY: cargo-build-armhf cargo-build-armel cargo-build-x64 build-docker-cargo \
		build-web build-archives build-archive-armhf build-archive-armel \
		build-archive-x64 release gh-release version clean

# For cross-compilation to ARM
cargo-build-armhf:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target arm-unknown-linux-gnueabihf

cargo-build-armel:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target arm-unknown-linux-gnueabi

# For cross-compilation to X86_64 Musl
cargo-build-x64:
	docker run -it \
	-v $(DIR)/target:/build \
	-v `pwd`:/root/project \
	radio-cross \
	cargo build --release --target x86_64-unknown-linux-gnu

build-docker-cargo:
	docker build . -t radio-cross:latest

# For building the UI
build-web:
	rm -rf ./radio-web/dist/
	cd ./radio-web/ && npm run build

# TODO: unify these build processes

# For building distributable archive files
build-archives: build-archive-x64 build-archive-armhf build-archive-armel

build-archive-armhf: build-web cargo-build-armhf
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

build-archive-armel: build-web cargo-build-armel
	mkdir -p ./$(BUILD_OUTPUT_DIR)/radio-web
	mkdir ./$(BUILD_OUTPUT_DIR)/images
	cp ./images/example.png ./$(BUILD_OUTPUT_DIR)/images/
	cp ./radio.service ./$(BUILD_OUTPUT_DIR)/
	cp ./install.sh ./$(BUILD_OUTPUT_DIR)/
	mkdir -p dist
	cp -r ./radio-web/dist/ ./$(BUILD_OUTPUT_DIR)/radio-web/
	cp ./target/arm-unknown-linux-gnueabi/release/radio ./$(BUILD_OUTPUT_DIR)/
	tar -cvzf dist/radio-$(VERSION)-arm-unknown-linux-gnueabi.tar.gz ./$(BUILD_OUTPUT_DIR)/
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

release: clean build-archives gh-release

# Publish the local release to Github releases
gh-release:
	gh release create v$(VERSION) ./dist/*.tar.gz -F ./CHANGELOG.md -t 'radio v$(version)'

version:
	python3 update_version.py

clean:
	rm -rf $(BUILD_OUTPUT_DIR)
	rm -rf ./radio-web/dist/
	rm -rf dist
