#include <stdlib.h>
#include <deislabs_ping_v01.h>

__attribute__((weak, export_name("canonical_abi_realloc")))
void *canonical_abi_realloc(
void *ptr,
size_t orig_size,
size_t org_align,
size_t new_size
) {
  void *ret = realloc(ptr, new_size);
  if (!ret)
  abort();
  return ret;
}

__attribute__((weak, export_name("canonical_abi_free")))
void canonical_abi_free(
void *ptr,
size_t size,
size_t align
) {
  free(ptr);
}
#include <string.h>

void deislabs_ping_v01_string_set(deislabs_ping_v01_string_t *ret, const char *s) {
  ret->ptr = (char*) s;
  ret->len = strlen(s);
}

void deislabs_ping_v01_string_dup(deislabs_ping_v01_string_t *ret, const char *s) {
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void deislabs_ping_v01_string_free(deislabs_ping_v01_string_t *ret) {
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
static int64_t RET_AREA[2];
__attribute__((export_name("ping")))
int32_t __wasm_export_deislabs_ping_v01_ping(int32_t arg, int32_t arg0) {
  deislabs_ping_v01_string_t arg1 = (deislabs_ping_v01_string_t) { (char*)(arg), (size_t)(arg0) };
  deislabs_ping_v01_string_t ret;
  deislabs_ping_v01_ping(&arg1, &ret);
  int32_t ptr = (int32_t) &RET_AREA;
  *((int32_t*)(ptr + 8)) = (int32_t) (ret).len;
  *((int32_t*)(ptr + 0)) = (int32_t) (ret).ptr;
  return ptr;
}
