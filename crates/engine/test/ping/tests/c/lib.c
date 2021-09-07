#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "deislabs_ping_v01.h"

void prepend(char *s, const char *t)
{
    size_t len = strlen(t);
    memmove(s + len, s, strlen(s) + 1);
    memcpy(s, t, len);
}

void deislabs_ping_v01_ping(
    deislabs_ping_v01_string_t *input,
    deislabs_ping_v01_string_t *output)
{
    char *out_msg = malloc(input->len + 6);
    strcpy(out_msg, input->ptr);
    prepend(out_msg, "PONG: ");

    output->len = input->len + 6;
    output->ptr = out_msg;
}
