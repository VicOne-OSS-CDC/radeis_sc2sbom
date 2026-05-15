CC = gcc
CFLAGS = -Wall -O2
LDFLAGS = -lssl -lcrypto -lpthread -lz

# Use pkg-config for OpenSSL
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl)
OPENSSL_LIBS = $(shell pkg-config --libs openssl)

all: myapp

myapp: main.o
	$(CC) -o myapp main.o $(LDFLAGS) $(OPENSSL_LIBS)

main.o: main.c
	$(CC) $(CFLAGS) $(OPENSSL_CFLAGS) -c main.c

clean:
	rm -f *.o myapp
