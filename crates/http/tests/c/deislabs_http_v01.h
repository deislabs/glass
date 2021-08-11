#ifndef __BINDINGS_DEISLABS_HTTP_V01_H
#define __BINDINGS_DEISLABS_HTTP_V01_H

#include <stdint.h>
#include <stdbool.h>

typedef struct {
  char *ptr;
  size_t len;
} deislabs_http_v01_string_t;

void deislabs_http_v01_string_set(deislabs_http_v01_string_t *ret, const char *s);
void deislabs_http_v01_string_dup(deislabs_http_v01_string_t *ret, const char *s);
void deislabs_http_v01_string_free(deislabs_http_v01_string_t *ret);
typedef uint16_t deislabs_http_v01_http_status_t;
typedef struct {
  uint8_t *ptr;
  size_t len;
} deislabs_http_v01_body_t;
void deislabs_http_v01_body_free(deislabs_http_v01_body_t *ptr);
typedef struct {
  deislabs_http_v01_string_t *ptr;
  size_t len;
} deislabs_http_v01_headers_t;
void deislabs_http_v01_headers_free(deislabs_http_v01_headers_t *ptr);
typedef struct {
  deislabs_http_v01_string_t *ptr;
  size_t len;
} deislabs_http_v01_params_t;
void deislabs_http_v01_params_free(deislabs_http_v01_params_t *ptr);
typedef deislabs_http_v01_string_t deislabs_http_v01_uri_t;
void deislabs_http_v01_uri_free(deislabs_http_v01_uri_t *ptr);
typedef uint8_t deislabs_http_v01_method_t;
#define DEISLABS_HTTP_V01_METHOD_GET 0
#define DEISLABS_HTTP_V01_METHOD_POST 1
#define DEISLABS_HTTP_V01_METHOD_PUT 2
#define DEISLABS_HTTP_V01_METHOD_DELETE 3
#define DEISLABS_HTTP_V01_METHOD_PATCH 4
typedef struct {
  // `true` if `val` is present, `false` otherwise
  bool tag;
  deislabs_http_v01_params_t val;
} deislabs_http_v01_option_params_t;
void deislabs_http_v01_option_params_free(deislabs_http_v01_option_params_t *ptr);
typedef struct {
  // `true` if `val` is present, `false` otherwise
  bool tag;
  deislabs_http_v01_body_t val;
} deislabs_http_v01_option_body_t;
void deislabs_http_v01_option_body_free(deislabs_http_v01_option_body_t *ptr);
typedef struct {
  deislabs_http_v01_method_t f0;
  deislabs_http_v01_uri_t f1;
  deislabs_http_v01_headers_t f2;
  deislabs_http_v01_option_params_t f3;
  deislabs_http_v01_option_body_t f4;
} deislabs_http_v01_request_t;
void deislabs_http_v01_request_free(deislabs_http_v01_request_t *ptr);
typedef struct {
  // `true` if `val` is present, `false` otherwise
  bool tag;
  deislabs_http_v01_headers_t val;
} deislabs_http_v01_option_headers_t;
void deislabs_http_v01_option_headers_free(deislabs_http_v01_option_headers_t *ptr);
typedef struct {
  deislabs_http_v01_http_status_t f0;
  deislabs_http_v01_option_headers_t f1;
  deislabs_http_v01_option_body_t f2;
} deislabs_http_v01_response_t;
void deislabs_http_v01_response_free(deislabs_http_v01_response_t *ptr);
void deislabs_http_v01_handler(deislabs_http_v01_request_t *req, deislabs_http_v01_http_status_t *ret0, deislabs_http_v01_option_headers_t *ret1, deislabs_http_v01_option_body_t *ret2);
#endif
