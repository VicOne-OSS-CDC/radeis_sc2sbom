AC_INIT([test-project], [1.0], [test@example.com])
AM_INIT_AUTOMAKE([-Wall -Werror foreign])

AC_PROG_CC

# Check for pthread library
AC_CHECK_LIB([pthread], [pthread_create])

# Check for math library
AC_SEARCH_LIBS([sqrt], [m])

# Check for SSL library
AC_CHECK_LIB([ssl], [SSL_new])

# Check for pkg-config packages
PKG_CHECK_MODULES([GLIB], [glib-2.0 >= 2.50])
PKG_CHECK_MODULES([OPENSSL], [openssl >= 3.0])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
