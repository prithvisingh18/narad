import socket
import threading

def handle_con(con, thread_no):
    try:
        # Send data (if needed)
        con.sendall("hello message from thread {}".format(thread_no).encode())
    except Exception as e:
        print(f"Error: {e}")
    
    finally:
        # Close the socket
        con.close()
    

def tcp_client():
    server_address = ("localhost", 9000)  # Replace with your server's IP address and port
    # client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        
    # try:
    #     # Connect to the server
    #     client_socket.connect(server_address)
        
    #     # Send data (if needed)
    #     for i in range(10):
    #         client_socket.sendall('{} messageno: {}\n'.format('node_greeting', i).encode())
        
    #     # Receive response
    #     response = client_socket.recv(1024)
    #     print(f"Response from server: {response.decode('utf-8')}")

    # except Exception as e:
    #     print(f"Error: {e}")
    
    # finally:
    #     # Close the socket
    #     client_socket.close()

    # num_connections = 10
    
    # for i in range(num_connections):
    #     # Create a socket
    #     client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    #     # Connect to the server
    #     client_socket.connect(server_address)

    #     threading.Thread(target=handle_con, args=(client_socket, i))

        
        

if __name__ == "__main__":
    tcp_client()
