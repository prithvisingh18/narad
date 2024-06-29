import socket
import time
import threading

def handle_server_data(sock):
    try:
        # Receive the greeting from the client
        greeting = sock.recv(256)
        if not greeting:
            print("Recived no greeting from client")
            return
        else:
            print("Recived greeting from client")
            print(greeting.decode())

        # Send a no authentication required response
        sock.sendall(b'\x05\x00')

        # Receive the connection request
        connection_request = sock.recv(256)
        if not connection_request:
            print("Recived no connection_request from client")
            return
        print("connection_request ->")
        print(connection_request)

        # Handling SOCKS5 request
        version, cmd, _, address_type = connection_request[:4]
        print(version, cmd, _, address_type)
        if cmd != 1:  # 1 = CONNECT
            sock.sendall(b'\x05\x07')  # Command not supported

        if address_type == 1:  # IPv4
            addr_ip = socket.inet_ntoa(connection_request[4:8])
            target_port = int.from_bytes(connection_request[8:10], 'big')
        elif address_type == 3:  # Domain name
            domain_length = connection_request[4]
            domain = connection_request[5:5+domain_length].decode()
            target_port = int.from_bytes(connection_request[5+domain_length:5+domain_length+2], 'big')
            addr_ip = socket.gethostbyname(domain)
        else:
            sock.sendall(b'\x05\x08')  # Address type not supported

        # Connect to the target server
        remote_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        remote_socket.connect((addr_ip, target_port))
        sock.sendall(b'\x05\x00\x00\x01' + socket.inet_aton('0.0.0.0') + (80).to_bytes(2, 'big'))

        # Relay data between the OG script and the target website
        relay_data(sock, remote_socket)
    except Exception as e:
        print(f"Error handling server data: {e}")
            
def relay_data(client_socket, server_socket):
    # Thread for client to server communication
    client_to_server = threading.Thread(target=forward_data, args=(client_socket, server_socket, "client->server"))
    # Thread for server to client communication
    server_to_client = threading.Thread(target=forward_data, args=(server_socket, client_socket, "server->client"))

    # Start both threads
    client_to_server.start()
    server_to_client.start()

    # Wait for both threads to complete
    client_to_server.join()
    server_to_client.join()

    # Close the sockets
    client_socket.close()
    server_socket.close()

def forward_data(source, destination, type):
    try:
        while True:
            data = source.recv(4096)
            if not data:
                continue
            print("Got data", type)
            destination.sendall(data)
    except Exception as e:
        print(f"Data forwarding error: {e} {type}")
    finally:
        if type == "client->server":
            destination.close()
        if type == "server->client":
            source.close()


def main():
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind(('0.0.0.0', 9001))
    server_socket.listen()
    print("Node listening on port 9001")
    while True:
        sock, add = server_socket.accept()
        print("Connected to controller from add ", add)
        threading.Thread(target=handle_server_data, args=(sock,)).start()
    

if __name__ == "__main__":
    main()
