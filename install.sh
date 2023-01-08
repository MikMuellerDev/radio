#!/bin/bash
# WARNING: This file should only be executed on the desired target machine

if [[ $EUID -eq 0 ]]; then
    echo "Please do not run this script as root"
    exit 1
fi

copy() {
    echo "$1 -> $2"
    sudo cp -r "$1" "$2" || exit 1
}

sudo mkdir -p /usr/bin/radio || exit 1

copy radio-web /usr/bin/radio
copy images/ /usr/bin/radio
copy radio /usr/bin/radio

sed -i "s/PLACEHOLDER/$USER/g" radio.service
copy radio.service /lib/systemd/system

sudo chown -R "$USER":"$USER" /usr/bin/radio

OLD_DIR=$(pwd)
cd /usr/bin/radio/ || exit 1
./radio
cd "$OLD_DIR" || exit 1

echo "Installation succeeded"
echo "NOTE: Execute 'sudo systemctl start radio' to start radio"
