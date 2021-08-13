#include <stdio.h>
#include <cassert>
#include <fstream>

#include <wizer.h>
#include <glass.h>

#include <deislabs_http_v01.h>

#include <js/Array.h>
#include <js/ArrayBuffer.h>
#include <js/ArrayBufferMaybeShared.h>

Glass::Runtime *runtime = nullptr;
JSContext *global_context;
static constexpr char DEFAULT_ENTRYPOINT_FILE[] = "index.js";
bool INITIALIZED = false;
std::string CODE;

void store_code_in_global()
{
    // Read `index.js` from the current directory and store its contents.
    std::ifstream in(DEFAULT_ENTRYPOINT_FILE);
    std::string str((std::istreambuf_iterator<char>(in)),
                    std::istreambuf_iterator<char>());

    CODE = str;
    INITIALIZED = true;
}

void init()
{
    assert(!INITIALIZED);

    JS_Init();
    global_context = JS_NewContext(JS::DefaultHeapMaxBytes);

    Glass::Runtime r = Glass::Runtime();

    if (!r.init_context(global_context))
        printf("init failed");
    runtime = &r;

    store_code_in_global();
    r.code = CODE;

    JSAutoRealm ar(global_context, runtime->global);

    if (!runtime->compile(global_context))
        runtime->abort(global_context, "evaluating JS");

    INITIALIZED = true;
}

void deislabs_http_v01_handler(
    deislabs_http_v01_request_t *req,
    deislabs_http_v01_http_status_t *status,
    deislabs_http_v01_option_headers_t *headers,
    deislabs_http_v01_option_body_t *body)
{
    if (!INITIALIZED)
    {
        store_code_in_global();
        assert(INITIALIZED);
    }

    JS_Init();
    global_context = JS_NewContext(JS::DefaultHeapMaxBytes);

    Glass::Runtime r = Glass::Runtime();

    if (!r.init_context(global_context))
        printf("init failed");
    runtime = &r;
    runtime->code = CODE;

    JSAutoRealm ar(global_context, runtime->global);

    if (!runtime->compile(global_context))
        runtime->abort(global_context, "evaluating JS");

    auto result = JS::RootedValue(global_context);

    auto js_body = JS::RootedValue(global_context);
    js_body.setObjectOrNull(JS::NewArrayBufferWithContents(global_context, req->f4.val.len, req->f4.val.ptr));

    auto js_method = JS::RootedValue(global_context);
    switch (req->f0)
    {

    case DEISLABS_HTTP_V01_METHOD_GET:
        js_method.setString(JS::RootedString(global_context, JS_NewStringCopyZ(global_context, "GET")));
        break;

    case DEISLABS_HTTP_V01_METHOD_POST:
        js_method.setString(JS::RootedString(global_context, JS_NewStringCopyZ(global_context, "POST")));
        break;

    case DEISLABS_HTTP_V01_METHOD_DELETE:
        js_method.setString(JS::RootedString(global_context, JS_NewStringCopyZ(global_context, "DELETE")));
        break;

    case DEISLABS_HTTP_V01_METHOD_PATCH:
        js_method.setString(JS::RootedString(global_context, JS_NewStringCopyZ(global_context, "PATCH")));
        break;
    }

    JS::RootedObject req_obj(global_context, JS_NewPlainObject(global_context));
    JS::RootedObject res_obj(global_context, JS_NewPlainObject(global_context));

    // TODO
    // Define property and insert headers in a JS map.
    JS_SetProperty(global_context, req_obj, "body", js_body);
    JS_SetProperty(global_context, req_obj, "method", js_method);

    JS::RootedValueArray<2> args(global_context);
    args[0].setObject(*req_obj);
    args[1].setObject(*res_obj);

    if (!JS_CallFunctionName(global_context, runtime->global, "handler", args, &result))
    {
        printf("cannot call handler");
    }

    do
    {
        runtime->process_pending_jobs(global_context);
    } while (js::HasJobsPending(global_context));

    JS::RootedValue rs(global_context);
    rs.setObject(*res_obj);

    JS::RootedValue res_status(global_context);
    JS_GetProperty(global_context, res_obj, "status", &res_status);
    *status = (uint16_t)res_status.toInt32();

    // TODO
    // Check if res.body is present, and try to read its value, in order,
    // as ArrayBuffer, Uint8Array, then string.
    JS::RootedValue res_body(global_context);
    if (!JS_GetProperty(global_context, res_obj, "body", &res_body))
        printf("cannot get response body");
}

WIZER_INIT(store_code_in_global);

int main(void)
{
    printf("entrypoint should not be used...\n");
}
