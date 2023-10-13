#!/usr/bin/env bash

# Modified from: https://github.com/puzzle/lightning-beer-tap/blob/master/dashboard/dashboard.sh

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

CHROME_ZOOM="2.0"

KIOSK_ARGS="--kiosk --disable-translate --incognito --force-device-scale-factor=${CHROME_ZOOM} ${DIR}/index.html"

# Don't sleep, don't blank, waste energy!

os_name=$(uname -s)

# Check if the system is Mac (Darwin) and execute the command accordingly
if [ "$os_name" = "Darwin" ]; then
    echo "Running on macOS"
    open -a 'Brave Browser' ${DIR}/index.html
elif [ "$os_name" = "Linux" ]; then
    DISPLAY=:0 xset s off
    DISPLAY=:0 xset s noblank

    DISPLAY=:0 nohup chromium-browser $KIOSK_ARGS &> /dev/null &
    echo "Running on Raspberry Pi (Raspberry Pi OS)"
else
    echo "Running on an unsupported operating system"
    # Add your command for unsupported systems here if needed
fi
