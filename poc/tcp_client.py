import socket

def tcp_client():
    server_address = ("localhost", 9999)  # Replace with your server's IP address and port
    num_connections = 10
    
    for _ in range(num_connections):
        # Create a socket
        client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        
        try:
            # Connect to the server
            client_socket.connect(server_address)
            
            # Send data (if needed)
            # client_socket.sendall(b'Hello, server!')
            
            # Receive response
            response = client_socket.recv(1024)
            print(f"Response from server: {response.decode('utf-8')}")

        except Exception as e:
            print(f"Error: {e}")
        
        finally:
            # Close the socket
            client_socket.close()

if __name__ == "__main__":
    tcp_client()
