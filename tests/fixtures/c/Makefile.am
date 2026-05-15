bin_PROGRAMS = myapp

myapp_SOURCES = main.c
myapp_LDADD = -lssl -lcrypto -lpthread
myapp_CFLAGS = $(GLIB_CFLAGS) $(OPENSSL_CFLAGS)
myapp_LDFLAGS = $(GLIB_LIBS) $(OPENSSL_LIBS)

lib_LTLIBRARIES = libfoo.la
libfoo_la_SOURCES = foo.c
libfoo_la_LIBADD = -lz
