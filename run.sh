#!/bin/bash

#Build the project in release mode
cargo build --release

#Check if the build was successful
ext=$?
echo "$ext"
if [[ $ext -ne 0 ]]; then
    echo "Build failed"
    exit $ext
fi

#Set the necessary capabilities on the binary
sudo setcap cap_net_admin=eip target/release/tcp

#run the binary in the background and store the PID
target/release/tcp & 
pid=$!

#Clear up any existing tun0 configuration
sudo ip link set dev tun0 down 2>/dev/null
sudo ip addr del 192.168.0.1/24 dev tun0 2>/dev/null

#Add the IP address and bring up the tun0 inteface
sudo ip addr add 192.168.0.1/24 dev tun0 
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid