#include <cassert>
#include <chrono>

#include <js/Array.h>
#include <js/CompilationAndEvaluation.h>
#include <js/Conversions.h>
#include <js/Initialization.h>
#include <js/SourceText.h>
#include <js/ValueArray.h>
#include <jsapi.h>
#include <js/TypeDecls.h>

#include <spidermonkey.h>

namespace Glass
{
    static JSClass global_class = {
        "global",
        JSCLASS_GLOBAL_FLAGS,
        &JS::DefaultGlobalClassOps};

    class Runtime
    {
    public:
        JS::PersistentRootedObject global;

        std::string code;
        JS::PersistentRooted<JS::Value> entrypoint;
        JS::PersistentRooted<JSObject *> event_data;

        bool init_context(JSContext *);
        bool define_builtins(JSContext *);
        bool compile(JSContext *);
        void abort(JSContext *, const char *description);
        bool report_unhandled_promise_rejections(JSContext *);
        void process_pending_jobs(JSContext *);

        bool print_stack(JSContext *, JS::HandleObject stack, FILE *fp);
        bool dump_value(JSContext *, JS::Value val, FILE *fp);
        JS::UniqueChars stringify_value(JSContext *, JS::HandleValue value);
        JS::UniqueChars encode(JSContext *, JS::HandleString str, size_t *encoded_len);
    };

    void rejection_tracker(JSContext *cx, bool mutedErrors, JS::HandleObject promise, JS::PromiseRejectionHandlingState state, void *data);
    bool js_main(JSContext *cx, unsigned argc, JS::Value *vp);
    void gc_callback(JSContext *cx, JSGCStatus status, JS::GCReason reason, void *data);

}
