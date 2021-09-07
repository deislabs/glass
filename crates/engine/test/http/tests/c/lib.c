#include <stdio.h>
#include "deislabs_http_v01.h"

void deislabs_http_v01_handler(
    deislabs_http_v01_request_t *req,
    deislabs_http_v01_http_status_t *status,
    deislabs_http_v01_option_headers_t *headers,
    deislabs_http_v01_option_body_t *body)
{
    *status = 418;

    char msg[34] = "Octavian was a pretty good emperor";

    body->tag = true;
    body->val.len = 34;
    body->val.ptr = msg;
}
