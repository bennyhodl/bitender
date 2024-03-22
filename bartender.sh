#!/usr/bin/env bash

# Modified from: https://github.com/puzzle/lightning-beer-tap/blob/master/application.sh

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

usage(){
cat << EOF                                                                              
Usage: $0 [OPTIONS]                                          

This script handles all interactions with the Bitcoin Bay Bartender.

OPTIONS:
	start	Initializes the device, starts the dashboard and bartender
	stop	Stops all services
	build	Build or rebuild the web socket to the lightning node 🦀
                                                                                           
EXAMPLES:                                                                               
    application restart

EOF
}

# Start all services on the beer tap device
app_start(){
  echo 🎵 hey, bartender 🎵

  rm -f ${DIR}/log.txt
	BARTENDER=${DIR}/target/debug/bartender
	# Check if the websocket bridge has been built
	if [ ! -f ${BARTENDER} ]; then
		app_build
	fi
	# Start up the dashboard
	source ${DIR}/dashboard.sh
	# Start websocket server, fork to background and pipe output to file
  export RUST_LOG=info
  nohup ${BARTENDER} --serve > log.txt 2>&1 &
  # cargo run
}

# Stop all services
app_stop(){
	echo "Killing all services..."
	killall bartender
	pkill -o chromium
	killall unclutter
}

# Build or rebuild the java lighning node web bridge
app_build(){
	echo "Building the bartender web socket server 🦀"
	cd ${DIR} && cargo build # >/dev/null 2>&1
}

# Argument parsing
case $1 in
	start)
	app_start
	exit 0;
	;;

	stop)
	app_stop
	exit 0;
	;;

	build)
	app_build
	exit 0;
	;;

	restart)
	app_stop
    app_start
	exit 0;
	;;

	*)
	usage
	exit 0;
	;;
esac
