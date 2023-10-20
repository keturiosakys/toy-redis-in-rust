#!/bin/bash

host="localhost"      # Replace with the hostname or IP address of the TCP server
port=6379                 # Replace with the port number of the TCP server

#for ((i=1; i<=10; i++))
#do
#    echo "Pinging TCP connection $i..."
#
#    # Use `nc` command to establish a TCP connection with the server
#    echo "PING" | redis-cli
#
#    # Alternatively, you can use `telnet` command
#    # telnet $host $port
#
#    echo "====================================="
#done

echo "ECHO hello" | redis-cli

