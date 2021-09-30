#ifndef __BINDINGS_DEISLABS_PING_V01_H
#define __BINDINGS_DEISLABS_PING_V01_H
#ifdef __cplusplus
extern "C"
{
  #endif
  
  #include <stdint.h>
  #include <stdbool.h>
  
  typedef struct {
    char *ptr;
    size_t len;
  } deislabs_ping_v01_string_t;
  
  void deislabs_ping_v01_string_set(deislabs_ping_v01_string_t *ret, const char *s);
  void deislabs_ping_v01_string_dup(deislabs_ping_v01_string_t *ret, const char *s);
  void deislabs_ping_v01_string_free(deislabs_ping_v01_string_t *ret);
  void deislabs_ping_v01_ping(deislabs_ping_v01_string_t *req, deislabs_ping_v01_string_t *ret0);
  #ifdef __cplusplus
}
#endif
#endif
