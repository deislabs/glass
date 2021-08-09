#ifdef __cplusplus
extern "C"
{
#endif

#ifndef __BINDINGS_GLASS_RUNTIME_H
#define __BINDINGS_GLASS_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>

  typedef struct
  {
    char *ptr;
    size_t len;
  } glass_runtime_string_t;

  void glass_runtime_string_set(glass_runtime_string_t *ret, const char *s);
  void glass_runtime_string_dup(glass_runtime_string_t *ret, const char *s);
  void glass_runtime_string_free(glass_runtime_string_t *ret);
  typedef uint16_t glass_runtime_http_status_t;
  typedef struct
  {
    uint8_t *ptr;
    size_t len;
  } glass_runtime_body_t;
  void glass_runtime_body_free(glass_runtime_body_t *ptr);
  typedef struct
  {
    glass_runtime_string_t *ptr;
    size_t len;
  } glass_runtime_headers_t;
  void glass_runtime_headers_free(glass_runtime_headers_t *ptr);
  typedef struct
  {
    glass_runtime_string_t *ptr;
    size_t len;
  } glass_runtime_params_t;
  void glass_runtime_params_free(glass_runtime_params_t *ptr);
  typedef glass_runtime_string_t glass_runtime_uri_t;
  void glass_runtime_uri_free(glass_runtime_uri_t *ptr);
  typedef uint8_t glass_runtime_method_t;
#define GLASS_RUNTIME_METHOD_GET 0
#define GLASS_RUNTIME_METHOD_POST 1
#define GLASS_RUNTIME_METHOD_PUT 2
#define GLASS_RUNTIME_METHOD_DELETE 3
#define GLASS_RUNTIME_METHOD_PATCH 4
  typedef struct
  {
    // `true` if `val` is present, `false` otherwise
    bool tag;
    glass_runtime_params_t val;
  } glass_runtime_option_params_t;
  void glass_runtime_option_params_free(glass_runtime_option_params_t *ptr);
  typedef struct
  {
    // `true` if `val` is present, `false` otherwise
    bool tag;
    glass_runtime_body_t val;
  } glass_runtime_option_body_t;
  void glass_runtime_option_body_free(glass_runtime_option_body_t *ptr);
  typedef struct
  {
    glass_runtime_method_t f0;
    glass_runtime_uri_t f1;
    glass_runtime_headers_t f2;
    glass_runtime_option_params_t f3;
    glass_runtime_option_body_t f4;
  } glass_runtime_request_t;
  void glass_runtime_request_free(glass_runtime_request_t *ptr);
  typedef struct
  {
    // `true` if `val` is present, `false` otherwise
    bool tag;
    glass_runtime_headers_t val;
  } glass_runtime_option_headers_t;
  void glass_runtime_option_headers_free(glass_runtime_option_headers_t *ptr);
  typedef struct
  {
    glass_runtime_http_status_t f0;
    glass_runtime_option_headers_t f1;
    glass_runtime_option_body_t f2;
  } glass_runtime_response_t;
  void glass_runtime_response_free(glass_runtime_response_t *ptr);
  void glass_runtime_handler(glass_runtime_request_t *req, glass_runtime_http_status_t *ret0, glass_runtime_option_headers_t *ret1, glass_runtime_option_body_t *ret2);

#ifdef __cplusplus
}
#endif
#endif
