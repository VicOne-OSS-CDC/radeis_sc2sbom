/* Phase 23 CWE-762 synthetic fixture - calloc + delete mismatch.
   Intentionally plain C so tree-sitter-c parses without error.
   The bad sink is the delete call paired with calloc() allocation. */
void cwe762_bad(void) {
    char *p = (char*)calloc(10, 1);
    delete p;
}
