/* Phase 11 fixture — CWE-134 negative case. */
#include <stdio.h>
void g(int x) {
    printf("hello world\n");
    fprintf(stderr, "x = %d\n", x);
}
