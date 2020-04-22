# Connecting To The Network

Technically speaking there are a near infinite amount of possible ways that a
user can connect to the network. Thanks to the near micro-service like approach
of the codebase the network is technically split in two:

  - Protocol-specific access points (APs)
  - Mitnick Network Engine ("The core")

The separation of APs and the core relieves the pressure on mitnick
supporting any specific protocol (e.g. `HTTP/S`, `SSH`, `TELNET`, `FTP`, `WS/S`).
This is because actually dealing with protocol specific behaviors is left to
the APs to implement it.

In theory you can use any of these protocols to access the network, just so
long as there is an AP running that supports it.

Currently these are the protocols implemented:

 - [TELNET](https://github.com/mental32/mitnick/blob/master/src/bin/telnet_server.rs)

## TELNET

The TELNET access point (AP) is probably the simplest protocol to use.
Connecting to the network and getting a live terminal is as simple as:

 - `telnet {address_or_domain_name}`

Technical details:

 - The AP isn't capable of handling generic TELNET sequences, currently these
   will just leak into the input of the users session.

 - Once connected the following sequences are immediately sent:
   1) `IAC . DO . SA` (suppress go ahead)
   2) `IAC . WONT . ECHO` (disable echoing input)
   3) `IAC . DO . LINEMODE` (send lines -> send chars)
