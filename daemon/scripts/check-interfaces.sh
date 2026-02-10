#!/bin/bash
# Helper script to display network interface information

echo "==================================="
echo "Fox Daemon - Network Interface Info"
echo "==================================="
echo ""

echo "Available Network Interfaces:"
echo "-----------------------------"
ip link show | grep -E "^[0-9]+:" | awk '{print $2}' | sed 's/://'

echo ""
echo "Detailed Interface Information:"
echo "-----------------------------"
ip -br addr show

echo ""
echo "Active Interfaces with IP:"
echo "-----------------------------"
ip -4 addr show | grep -E "^[0-9]+:|inet " | awk '/^[0-9]+:/ {iface=$2} /inet / {print iface, $2}'

echo ""
echo "Recommended interface for monitoring:"
echo "-----------------------------"

# Find the default route interface
DEFAULT_IFACE=$(ip route | grep default | awk '{print $5}' | head -n 1)

if [ -n "$DEFAULT_IFACE" ]; then
    echo "$DEFAULT_IFACE (used for default route)"
    echo ""
    echo "To use this interface, set in config.toml:"
    echo "[daemon]"
    echo "interface = \"$DEFAULT_IFACE\""
else
    echo "No default route found. Please choose an interface from the list above."
fi

echo ""
echo "To test packet capture on an interface:"
echo "sudo tcpdump -i <interface> -c 10 arp"
echo ""
