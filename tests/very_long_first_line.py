from socket import socket
import time

cli = socket()
cli.connect(("127.0.0.1", 8080))

cli.sendall(("HelloWorld" * 12000 + "\r\n").encode())

while True:
    try:
        data = cli.recv(1024)
        if data:
            print(data.decode())
    except Exception:
        break

    time.sleep(1)
