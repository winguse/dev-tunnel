# dev-tunnel

## Overview

`dev-tunnel` is a VPN software project that allows clients to create a utun device and connect to a server using WebSocket. The server operates in user mode and does not create any utun or tap devices. It reads network layer 3 packets from the client over the WebSocket connection and, if the packets are TCP, UDP, or ICMP, it creates corresponding sessions to translate the packets in both directions.
