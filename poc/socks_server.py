import socket
import threading

def handle_client_connection(client_socket):
    try:
        # Receive the greeting from the client
        greeting = client_socket.recv(256)
        if not greeting:
            print("Recived no greeting from client")
            return
        else:
            print("Recived greeting from client")
            print(greeting.decode())

        # Send a no authentication required response
        client_socket.sendall(b'\x05\x00')

        # Receive the connection request
        connection_request = client_socket.recv(256)
        if not connection_request:
            print("Recived no connection_request from client")
            return
        print(connection_request)
        # Parse the request
        version, cmd, _, address_type = connection_request[:4]
        print("Recived connection request -> ")
        print(version, cmd, _, address_type)
        if cmd != 1:  # 1 = CONNECT
            client_socket.sendall(b'\x05\x07')  # Command not supported
            return

        if address_type == 1:  # IPv4
            addr_ip = socket.inet_ntoa(connection_request[4:8])
            target_port = int.from_bytes(connection_request[8:10], 'big')
        elif address_type == 3:  # Domain name
            domain_length = connection_request[4]
            domain = connection_request[5:5+domain_length].decode()
            target_port = int.from_bytes(connection_request[5+domain_length:5+domain_length+2], 'big')
            addr_ip = socket.gethostbyname(domain)
            print(domain_length, domain, target_port, addr_ip)
        else:
            client_socket.sendall(b'\x05\x08')  # Address type not supported
            return

        # Connect to the target server
        remote_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        remote_socket.connect((addr_ip, target_port))

        # Send success reply to the client
        reply = b'\x05\x00\x00\x01'
        reply += socket.inet_aton('0.0.0.0') + (80).to_bytes(2, 'big')  # Bind port and address not used in CONNECT
        print(reply)
        client_socket.sendall(reply)

        # Start relaying between client and target
        relay_data(client_socket, remote_socket)
    except Exception as e:
        print(f"Error handling client connection: {e}")
    finally:
        print("Closing client connection")
        client_socket.close()

def forward_data(source, destination):
    try:
        while True:
            data = source.recv(4096)
            if not data:
                print("Received no data; closing the relay.")
                break
            destination.sendall(data)
    except Exception as e:
        print(f"Relay error: {e}")
    finally:
        # Properly shutdown the socket to send a TCP FIN signal
        source.shutdown(socket.SHUT_RD)
        destination.shutdown(socket.SHUT_WR)

def relay_data(client_socket, server_socket):
    # Thread for client to server communication
    client_to_server = threading.Thread(target=forward_data, args=(client_socket, server_socket))
    # Thread for server to client communication
    server_to_client = threading.Thread(target=forward_data, args=(server_socket, client_socket))

    # Start both threads
    client_to_server.start()
    server_to_client.start()

    # Wait for both threads to complete
    client_to_server.join()
    server_to_client.join()

    # Close the sockets
    client_socket.close()
    server_socket.close()

def main():
    proxy_ip = '0.0.0.0'
    proxy_port = 1081  # Standard SOCKS port

    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind((proxy_ip, proxy_port))
    server_socket.listen(50)
    print(f"SOCKS5 Proxy Server listening on {proxy_ip}:{proxy_port}")

    try:
        while True:
            client_socket, addr = server_socket.accept()
            print(f"Accepted connection from {addr}")
            client_thread = threading.Thread(target=handle_client_connection, args=(client_socket,))
            client_thread.start()
    except KeyboardInterrupt:
        print("Shutting down the proxy server.")
    finally:
        server_socket.close()

if __name__ == "__main__":
    main()

