import socket
import threading

def client_handler(client_socket, node_socket):
    try:
        greeting = client_socket.recv(256)
        if not greeting:
            print("Recived no greeting from client")
            return
        else:
            print("Recived greeting from client")
            print(greeting.decode())
        node_socket.sendall(greeting)

        # Receive the connection request
        connection_request = client_socket.recv(256)
        if not connection_request:
            print("Recived no connection_request from client")
            return
        print("connection_request ->")
        print(connection_request)
        node_socket.sendall(connection_request)
        
        while True:
            data = client_socket.recv(4096)
            if not data:
                continue
            print("Received data from client forwarding it to the node.")
            node_socket.sendall(data)  # Send client data to the Node Script
    except Exception as e:
        print(f"Error handling client data: {e}")

def node_handler(node_socket, client_socket):
    try:
        while True:
            data = node_socket.recv(4096)
            if not data:
                continue
            # if data == b'heartbeat':
            #     print("Received heartbeat")
            else:
                print("Received response data from proxy")
                client_socket.sendall(data)  # Send response data to the client
    except Exception as e:
        print(f"Error handling node data: {e}")

# def heartbeat_handler(node_socket):
#     try:
#         while True:
#             data = node_socket.recv(4096)
#             if data == b'heartbeat':
#                 print("Received heartbeat")
#             continue
#     except Exception as e:
#         print(f"Error handling node data: {e}")

def accept_connections(server_socket):
    # node_socket, add = server_socket.accept()  # First connection is from the node
    # print(f"Node connected from address -> ${add}")
    # threading.Thread(target=heartbeat_handler, args=(node_socket,)).start()
    
    while True:
        # Accept client connection
        client_socket, addr = server_socket.accept()
        print(f"Client connected from {addr}")

        # Create a connection to node
        node_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        node_socket.connect(('localhost', 9001))
        print("Connected to node")
        
        # Start threads for handling data to and from the node
        threading.Thread(target=client_handler, args=(client_socket, node_socket)).start()
        threading.Thread(target=node_handler, args=(node_socket, client_socket)).start()

def main():
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind(('0.0.0.0', 9000))
    server_socket.listen()
    print("Controller listening on port 9000")
    accept_connections(server_socket)
    server_socket.close()
    # with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as server_socket:
    #     server_socket.bind(('0.0.0.0', 9000))
    #     server_socket.listen()
    #     print("OG Script listening on port 9000")
    #     accept_connections(server_socket)

if __name__ == "__main__":
    main()
