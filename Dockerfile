# Cross compilation environment for radio
# Most of the configuration is taken from 'https://github.com/librespot-org/librespot/blob/dev/contrib/Dockerfile'
FROM debian:stretch

RUN dpkg --add-architecture armhf\
    && dpkg --add-architecture armel

RUN apt-get update

RUN apt-get install -y\
    curl git\
    build-essential pkg-config crossbuild-essential-armhf crossbuild-essential-armel\
    libasound2-dev libasound2-dev:armhf libasound2-dev:armel

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin/:${PATH}"
RUN rustup target add arm-unknown-linux-gnueabihf\
    && rustup target add arm-unknown-linux-gnueabi\
    && rustup target add x86_64-unknown-linux-gnu

RUN mkdir /.cargo && \
    echo '[target.arm-unknown-linux-gnueabihf]\nlinker = "arm-linux-gnueabihf-gcc"' >> /.cargo/config\
    && echo '[target.arm-unknown-linux-gnueabi]\nlinker = "arm-linux-gnueabi-gcc"' >> /.cargo/config
    #echo '[target.x86_64-unknown-linux-gnu]\nlinker = "musl-gcc"' >> /.cargo/config

RUN mkdir /build && \
    mkdir /pi-tools && \
    curl -L https://github.com/raspberrypi/tools/archive/648a6eeb1e3c2b40af4eb34d88941ee0edeb3e9a.tar.gz | tar xz --strip-components 1 -C /pi-tools

ENV CARGO_TARGET_DIR /build
ENV CARGO_HOME /build/cache
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH_arm-unknown-linux-gnueabihf=/usr/lib/arm-linux-gnueabihf/pkgconfig/
ENV PKG_CONFIG_PATH_arm-unknown-linux-gnueabi=/usr/lib/arm-linux-gnueabi/pkgconfig/
#ENV PKG_CONFIG_PATH_x86_64-unknown-linux-musl=/usr/lib/x86_64-linux-musl/pkgconfig/

RUN mkdir /root/project
WORKDIR /root/project
