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
    auto arg = JS::RootedValue(global_context);
    auto abuf = JS::NewArrayBufferWithContents(global_context, req->f4.val.len, req->f4.val.ptr);
    arg.setObject(*abuf);
    auto argsv = JS::HandleValueArray(arg);

    if (!JS_CallFunctionName(global_context, runtime->global, "handler", argsv, &result))
    {
        printf("cannot call handler");
    }

    do
    {
        runtime->process_pending_jobs(global_context);
    } while (js::HasJobsPending(global_context));

    *status = 404;
}

void test()
{
    auto x = JS::NewArrayBuffer(global_context, 0);
}

// static JSBool my_array_dump(JSContext *cx, uintN argc, jsval *vp)
// {
//     JSObject *obj;
//     JS_ValueToObject(cx, vp[0 + 2], &obj);
//     js::ArrayBuffer *A;
//     A = js::ArrayBuffer::fromJSObject(obj);
//     int *B = (int *)A->data;
//     for (int i = 0; i < A->byteLength / 4; i++)
//         printf("%i ", B[i]);
//     return JS_TRUE;
// }

WIZER_INIT(store_code_in_global);

int main(void)
{
    printf("entrypoint should not be used...\n");
}
