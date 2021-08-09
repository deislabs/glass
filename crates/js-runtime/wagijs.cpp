#include <cassert>
#include <chrono>
#include <fstream>

#include <js/Array.h>
#include <js/CompilationAndEvaluation.h>
#include <js/Conversions.h>
#include <js/Initialization.h>
#include <js/SourceText.h>
#include <js/ValueArray.h>
#include <jsapi.h>
#include <js/TypeDecls.h>

#include <builtins.h>
#include <spidermonkey.h>
#include <wagijs.h>
// #include <wasi/libc-environ.h>
// #include <wizer.h>

namespace WagiJS
{
    JS::PersistentRootedObject uhrp;

    bool Runtime::init_context(JSContext *ctx)
    {
        // JS_Init();
        // ctx = JS_NewContext(JS::DefaultHeapMaxBytes);
        if (!ctx)
            return false;
        if (!js::UseInternalJobQueues(ctx) || !JS::InitSelfHostedCode(ctx))
            return false;

        JS::RealmOptions options;
        options.creationOptions()
            .setStreamsEnabled(true)
            .setReadableByteStreamsEnabled(true)
            .setBYOBStreamReadersEnabled(true)
            .setReadableStreamPipeToEnabled(true)
            .setWritableStreamsEnabled(true)
            .setIteratorHelpersEnabled(true)
            .setWeakRefsEnabled(JS::WeakRefSpecifier::EnabledWithoutCleanupSome);

        JS::DisableIncrementalGC(ctx);

        JS::RootedObject g(ctx, JS_NewGlobalObject(ctx, &global_class, nullptr, JS::FireOnNewGlobalHook,
                                                   options));
        if (!g)
            return false;

        JSAutoRealm ar(ctx, g);
        if (!JS::InitRealmStandardClasses(ctx))
            return false;

        JS::SetPromiseRejectionTrackerCallback(ctx, rejection_tracker);

        global.init(ctx, g);
        uhrp.init(ctx, JS::NewSetObject(ctx));
        if (!uhrp)
            return false;

        if (!define_builtins(ctx))
        {
            printf("cannot define builtins");
            return false;
        }

        return true;
    }

    bool Runtime::compile(JSContext *ctx)
    {
        JSAutoRealm ar(ctx, global);

        char *cstr = new char[code.length() + 1];
        std::strcpy(cstr, code.c_str());

        JS::CompileOptions opts(ctx);
        opts.setForceFullParse();
        opts.setFileAndLine("<stdin>", 1);

        JS::SourceText<mozilla::Utf8Unit> srcBuf;
        if (!srcBuf.init(ctx, cstr, strlen(cstr), JS::SourceOwnership::TakeOwnership))
        {
            return false;
        }

        JS::RootedScript script(ctx);
        {
            JS::AutoDisableGenerationalGC noGGC(ctx);
            script = JS::Compile(ctx, opts, srcBuf);
            if (!script)
                return false;
        }

        JS::RootedValue result(ctx);
        JS::PrepareForFullGC(ctx);
        JS::NonIncrementalGC(ctx, JS::GCOptions::Shrink, JS::GCReason::API);

        if (!JS_ExecuteScript(ctx, script, &result))
            return false;

        JS::PrepareForFullGC(ctx);
        JS::NonIncrementalGC(ctx, JS::GCOptions::Normal, JS::GCReason::API);

        JS_SetGCCallback(ctx, gc_callback, nullptr);

        if (!result.isUndefined())
            if (!dump_value(ctx, result, stdout))
                exit(1);

        return true;
    }

    void Runtime::process_pending_jobs(JSContext *ctx)
    {
        JSAutoRealm ar(ctx, global);

        while (js::HasJobsPending(ctx))
        {
            js::RunJobs(ctx);

            if (JS_IsExceptionPending(ctx))
                printf("Cannot process pending promises");
        }
    }

    void Runtime::abort(JSContext *ctx, const char *description)
    {
        if (JS_IsExceptionPending(ctx))
        {
            JS::ExceptionStack exception(ctx);
            if (!JS::GetPendingExceptionStack(ctx, &exception))
            {
                fprintf(stderr, "Error: exception pending after %s, but got another error "
                                "when trying to retrieve it. Aborting.\n",
                        description);
            }
            else
            {
                fprintf(stderr, "Exception while %s: ", description);
                dump_value(ctx, exception.exception(), stderr);
                print_stack(ctx, exception.stack(), stderr);
            }
        }
        else
        {
            fprintf(stderr, "Error while %s, but no exception is pending. "
                            "Aborting, since that doesn't seem recoverable at all.\n",
                    description);
        }

        if (JS::SetSize(ctx, uhrp) > 0)
        {
            fprintf(stderr,
                    "Additionally, some promises were rejected, but the rejection never handled:\n");
            report_unhandled_promise_rejections(ctx);
        }

        fflush(stderr);
        exit(1);
    }

    bool Runtime::print_stack(JSContext *ctx, JS::HandleObject stack, FILE *fp)
    {
        JS::RootedString stackStr(ctx);
        if (!BuildStackString(ctx, nullptr, stack, &stackStr, 2))
        {
            return false;
        }
        size_t stack_len;

        JS::UniqueChars utf8chars = encode(ctx, stackStr, &stack_len);
        if (!utf8chars)
            return false;
        fprintf(fp, "%s\n", utf8chars.get());
        return true;
    }

    JS::UniqueChars Runtime::encode(JSContext *ctx, JS::HandleString str, size_t *encoded_len)
    {
        JS::UniqueChars text = JS_EncodeStringToUTF8(ctx, str);
        if (!text)
            return nullptr;

        JSLinearString *linear = JS_EnsureLinearString(ctx, str);
        *encoded_len = JS::GetDeflatedUTF8StringLength(linear);
        return text;
    }

    bool Runtime::dump_value(JSContext *ctx, JS::Value val, FILE *fp)
    {
        JS::RootedValue value(ctx, val);
        JS::UniqueChars utf8chars = stringify_value(ctx, value);
        if (!utf8chars)
            return false;
        fprintf(fp, "%s\n", utf8chars.get());
        return true;
    }

    JS::UniqueChars Runtime::stringify_value(JSContext *ctx, JS::HandleValue value)
    {
        JS::RootedString str(ctx, JS_ValueToSource(ctx, value));
        if (!str)
            return nullptr;

        return JS_EncodeStringToUTF8(ctx, str);
    }

    bool Runtime::report_unhandled_promise_rejections(JSContext *ctx)
    {
        // JSAutoRealm ar(ctx, global);

        JS::RootedValue iterable(ctx);
        if (!JS::SetValues(ctx, uhrp, &iterable))
            return false;

        JS::ForOfIterator it(ctx);
        if (!it.init(iterable))
            return false;

        JS::RootedValue promise_val(ctx);
        JS::RootedObject promise(ctx);
        while (true)
        {
            bool done;
            if (!it.next(&promise_val, &done))
                return false;

            if (done)
                break;

            promise = &promise_val.toObject();
            fprintf(stderr, "Promise rejected but never handled: ");
            dump_value(ctx, JS::GetPromiseResult(promise), stderr);
        }

        return true;
    }

    bool Runtime::define_builtins(JSContext *ctx)
    {
        JSAutoRealm ar(ctx, global);
        if (!Console::create(ctx, global))
            return false;

        if (!TextEncoder::init_class(ctx, global))
            return false;

        if (!TextDecoder::init_class(ctx, global))
            return false;

        return true;
    }

    void gc_callback(JSContext *ctx, JSGCStatus status, JS::GCReason reason, void *data)
    {
        printf("gc for reason %s, %s\n", JS::ExplainGCReason(reason), status ? "end" : "start");
    }

    void rejection_tracker(JSContext *ctx, bool mutedErrors, JS::HandleObject promise,
                           JS::PromiseRejectionHandlingState state, void *data)
    {
        JS::RootedValue promiseVal(ctx, JS::ObjectValue(*promise));

        switch (state)
        {
        case JS::PromiseRejectionHandlingState::Unhandled:
        {
            if (!JS::SetAdd(ctx, uhrp, promiseVal))
            {
                fprintf(stderr, "Adding an unhandled rejected promise to the promise "
                                "rejection tracker failed");
            }
            return;
        }
        case JS::PromiseRejectionHandlingState::Handled:
        {
            bool deleted = false;
            if (!JS::SetDelete(ctx, uhrp, promiseVal, &deleted))
            {
                fprintf(stderr, "Removing an handled rejected promise from the promise "
                                "rejection tracker failed");
            }
        }
        }
    }
}
