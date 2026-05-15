/* Phase 11 fixture — one call per CWE rule. Compilation not required. */
#include <stdio.h>
#include <string.h>
void f(char *user_input, char *d, char *s, int n) {
    strcpy(d, s);                     /* CWE-120 */
    system(user_input);               /* CWE-78 */
    gets(d);                          /* CWE-242, CWE-676 */
    MD5((unsigned char *)d, n, NULL); /* CWE-327 */
    tmpnam(d);                        /* CWE-377 */
    malloc(n);                        /* CWE-190 */
    printf(user_input);               /* CWE-134 (no literal) */
    getenv("HOME");                   /* CWE-807 */
    access(s, 0);                     /* CWE-362, CWE-367 */
    strlen(s);                        /* CWE-126 */
    scanf("%s", d);                   /* CWE-676 */
}
