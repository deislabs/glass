#include <stdlib.h>
#include <stdio.h>
#include <glass_runtime.h>

__attribute__((weak, export_name("canonical_abi_realloc"))) void *canonical_abi_realloc(
    void *ptr,
    size_t orig_size,
    size_t org_align,
    size_t new_size)
{
  void *ret = realloc(ptr, new_size);
  if (!ret)
    abort();
  return ret;
}

__attribute__((weak, export_name("canonical_abi_free"))) void canonical_abi_free(
    void *ptr,
    size_t size,
    size_t align)
{
  free(ptr);
}
#include <string.h>

void glass_runtime_string_set(glass_runtime_string_t *ret, const char *s)
{
  ret->ptr = (char *)s;
  ret->len = strlen(s);
}

void glass_runtime_string_dup(glass_runtime_string_t *ret, const char *s)
{
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void glass_runtime_string_free(glass_runtime_string_t *ret)
{
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
void glass_runtime_body_free(glass_runtime_body_t *ptr)
{
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void glass_runtime_headers_free(glass_runtime_headers_t *ptr)
{
  for (size_t i = 0; i < ptr->len; i++)
  {
    glass_runtime_string_free(&ptr->ptr[i]);
  }
  canonical_abi_free(ptr->ptr, ptr->len * 8, 4);
}
void glass_runtime_params_free(glass_runtime_params_t *ptr)
{
  for (size_t i = 0; i < ptr->len; i++)
  {
    glass_runtime_string_free(&ptr->ptr[i]);
  }
  canonical_abi_free(ptr->ptr, ptr->len * 8, 4);
}
void glass_runtime_uri_free(glass_runtime_uri_t *ptr)
{
  canonical_abi_free(ptr->ptr, ptr->len * 1, 1);
}
void glass_runtime_option_params_free(glass_runtime_option_params_t *ptr)
{
  switch (ptr->tag)
  {
  case 1:
  {
    glass_runtime_params_free(&ptr->val);
    break;
  }
  }
}
void glass_runtime_option_body_free(glass_runtime_option_body_t *ptr)
{
  switch (ptr->tag)
  {
  case 1:
  {
    glass_runtime_body_free(&ptr->val);
    break;
  }
  }
}
void glass_runtime_request_free(glass_runtime_request_t *ptr)
{
  glass_runtime_uri_free(&ptr->f1);
  glass_runtime_headers_free(&ptr->f2);
  glass_runtime_option_params_free(&ptr->f3);
  glass_runtime_option_body_free(&ptr->f4);
}
void glass_runtime_option_headers_free(glass_runtime_option_headers_t *ptr)
{
  switch (ptr->tag)
  {
  case 1:
  {
    glass_runtime_headers_free(&ptr->val);
    break;
  }
  }
}
void glass_runtime_response_free(glass_runtime_response_t *ptr)
{
  glass_runtime_option_headers_free(&ptr->f1);
  glass_runtime_option_body_free(&ptr->f2);
}
static int64_t RET_AREA[7];
__attribute__((export_name("handler")))
int32_t
__wasm_export_glass_runtime_handler(int32_t arg, int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5, int32_t arg6, int32_t arg7, int32_t arg8, int32_t arg9)
{
  glass_runtime_option_params_t variant;
  variant.tag = arg4;
  switch ((int32_t)variant.tag)
  {
  case 0:
  {
    break;
  }
  case 1:
  {
    variant.val = (glass_runtime_params_t){(glass_runtime_string_t *)(arg5), (size_t)(arg6)};
    break;
  }
  }
  glass_runtime_option_body_t variant10;
  variant10.tag = arg7;
  switch ((int32_t)variant10.tag)
  {
  case 0:
  {
    break;
  }
  case 1:
  {
    variant10.val = (glass_runtime_body_t){(uint8_t *)(arg8), (size_t)(arg9)};
    break;
  }
  }
  glass_runtime_request_t arg11 = (glass_runtime_request_t){
      arg,
      (glass_runtime_uri_t){(char *)(arg0), (size_t)(arg1)},
      (glass_runtime_headers_t){(glass_runtime_string_t *)(arg2), (size_t)(arg3)},
      variant,
      variant10,
  };
  glass_runtime_http_status_t ret;
  glass_runtime_option_headers_t ret12;
  glass_runtime_option_body_t ret13;
  glass_runtime_handler(&arg11, &ret, &ret12, &ret13);
  int32_t variant15;
  int32_t variant16;
  int32_t variant17;
  switch ((int32_t)(((glass_runtime_response_t){ret, ret12, ret13}).f1).tag)
  {
  case 0:
  {
    variant15 = 0;
    variant16 = 0;
    variant17 = 0;
    break;
  }
  case 1:
  {
    const glass_runtime_headers_t *payload14 = &(((glass_runtime_response_t){ret, ret12, ret13}).f1).val;
    variant15 = 1;
    variant16 = (int32_t)(*payload14).ptr;
    variant17 = (int32_t)(*payload14).len;
    break;
  }
  }
  int32_t variant20;
  int32_t variant21;
  int32_t variant22;
  switch ((int32_t)(((glass_runtime_response_t){ret, ret12, ret13}).f2).tag)
  {
  case 0:
  {
    variant20 = 0;
    variant21 = 0;
    variant22 = 0;
    break;
  }
  case 1:
  {
    const glass_runtime_body_t *payload19 = &(((glass_runtime_response_t){ret, ret12, ret13}).f2).val;
    variant20 = 1;
    variant21 = (int32_t)(*payload19).ptr;
    variant22 = (int32_t)(*payload19).len;
    break;
  }
  }
  int32_t ptr = (int32_t)&RET_AREA;
  *((int32_t *)(ptr + 48)) = variant22;
  *((int32_t *)(ptr + 40)) = variant21;
  *((int32_t *)(ptr + 32)) = variant20;
  *((int32_t *)(ptr + 24)) = variant17;
  *((int32_t *)(ptr + 16)) = variant16;
  *((int32_t *)(ptr + 8)) = variant15;
  *((int32_t *)(ptr + 0)) = (int32_t)(((glass_runtime_response_t){ret, ret12, ret13}).f0);
  return ptr;
}

__attribute__((weak)) void
glass_runtime_handler(glass_runtime_request_t *req, glass_runtime_http_status_t *ret0, glass_runtime_option_headers_t *ret1, glass_runtime_option_body_t *ret2)
{
  printf("Handling request...\n");
  *ret0 = 46;
}
