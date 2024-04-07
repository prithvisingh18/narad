# Narad

The project aims to develop a SOCKS proxy, a protocol for handling network requests between a client and a server through an intermediary. This project serves as a deep dive into network protocols, providing a hands-on approach to understanding and manipulating TCP/IP communications.


- [x] Setup TCP server, which accepts a connection and simply returns "Recieved".
- [ ] Build a HTTP client which connects to a http server.
- [ ] Try to connect to TCP server using a http client, to understand how HTTP communication happen in TCP.
- [ ] Set up HTTP server.
- [ ] Build a TCP client.
- [ ] Try to replicate a HTTP client in using TCP.
- [ ] Connect TCP client and server, which listens for TCP packets from 1 client and sends it to another predefined server, then relays the recived packets back to the client.
- [ ] Now implement CONNECT and BIND in the above server-client combination, where in the server gets the destination address in the BIND packet. 