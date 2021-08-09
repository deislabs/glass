#include <glass_runtime.h>
#include <stdio.h>

#include <cassert>
#include <fstream>

#include <wizer.h>
#include <wagijs.h>

WagiJS::Runtime *runtime = nullptr;
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

    WagiJS::Runtime r = WagiJS::Runtime();

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

void glass_runtime_handler(glass_runtime_request_t *req, glass_runtime_http_status_t *ret0, glass_runtime_option_headers_t *ret1, glass_runtime_option_body_t *ret2)
{

    // __wasilibc_initialize_environ();

    printf("Handling request... in JS runtime...\n");

    if (!INITIALIZED)
    {
        store_code_in_global();
        assert(INITIALIZED);
    }

    JS_Init();
    global_context = JS_NewContext(JS::DefaultHeapMaxBytes);

    WagiJS::Runtime r = WagiJS::Runtime();

    if (!r.init_context(global_context))
        printf("init failed");
    runtime = &r;
    runtime->code = CODE;

    JSAutoRealm ar(global_context, runtime->global);

    if (!runtime->compile(global_context))
        runtime->abort(global_context, "evaluating JS");

    JS::RootedValue result(global_context);
    if (!JS_CallFunctionName(global_context, runtime->global, "main", JS::HandleValueArray::empty(), &result))
    {
        printf("cannot call entrypoint");
    }

    JS::RootedValue result2(global_context);
    JS::RootedValue val(global_context);
    val.setNumber(46);
    JS::HandleValueArray argsv = JS::HandleValueArray(val);

    if (!JS_CallFunctionName(global_context, runtime->global, "test", argsv, &result2))
    {
        printf("cannot call entrypoint");
    }

    do
    {
        runtime->process_pending_jobs(global_context);
    } while (js::HasJobsPending(global_context));

    *ret0 = 404;
}

WIZER_INIT(store_code_in_global);

int main(void)
{
    printf("entrypoint should not be used...\n");
}
