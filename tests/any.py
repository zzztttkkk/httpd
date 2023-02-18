from socket import socket
import time

cli = socket()
cli.connect(("127.0.0.1", 8080))

cli.sendall(b"GET / HTTP/1.0\r\nBadValue: \x1e\x12\xee\xad\r\n\r\n")

while True:
    try:
        data = cli.recv(1024)
        if data:
            print(data.decode())
    except Exception:
        break

    time.sleep(1)
