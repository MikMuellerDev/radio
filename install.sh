#!/bin/bash
# WARNING: This file should only be executed on the desired target machine

if [ "$EUID" -ne 0 ]; then
    echo "Please run this script as root or using sudo"
    exit 1
fi

copy() {
    echo "$1 -> $2"
    cp -r "$1" "$2" || exit 1
}

./radio
mkdir -p /usr/bin/radio || exit 1

copy radio-web /usr/bin/radio
copy images/ /usr/bin/radio

copy radio /usr/bin/radio
copy config.toml /usr/bin/radio
copy radio.service /lib/systemd/system

chmod o+t /usr/bin/radio/

echo "Installation succeeded"
echo "NOTE: Execute 'sudo systemctl start radio' to start radio"
