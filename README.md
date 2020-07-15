# Tapdev

## What?

A small linux-only program that creates a TAP-device and opens a serial-port. All data coming from the TAP-device
is written to the serial-port and all data read from the serial port is written to the TAP-device.

At the moment this can be used to establish a bridge between two computers:

```shell script
# computer A, assumed to have an IP address 192.168.0.1:
$ socat -d -d tcp-listen:12345 pty,raw,echo=0,link=$HOME/dev/ttyV0
$ sudo ./tapdev $HOME/dev/ttyV0
$ sudo ip addr add 10.0.0.1/24 dev tap0
$ sudo ip link set tap0 up

# computer B, must be able to connect to the IP of 192.168.0.1:
$ socat -d -d pty,raw,echo=0,link=$HOME/dev/ttyV0 tcp:192.168.0.1:12345
$ sudo ./tapdev $HOME/dev/ttyV0
$ sudo ip addr add 10.0.0.2/24 dev tap0
$ sudo ip link set tap0 up

# computer A can ping computer B via the tap device
$ ping 10.0.0.2
# computer B can ping computer A via the tap device
$ ping 10.0.0.1
```

## Why?

The end goal is to pair this utility with an embedded board ([nrf52840-DK](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK))
and use the short-range radio on that device to communicate between the computers (the communication between the board
and the computer would be via a serial port), instead of using `socat`.