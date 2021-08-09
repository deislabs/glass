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
// #include <wasi/libc-environ.h>
// #include <wizer.h>

#include <js/ArrayBuffer.h>
#include <js/experimental/TypedData.h>
#include <js/JSON.h>
#include <js/shadow/Object.h>
#include <js/Stream.h>
#include <js/Value.h>

namespace Console
{
    JS::UniqueChars encode_log(JSContext *cx, JS::HandleString str, size_t *encoded_len);
    JS::UniqueChars encode_log(JSContext *cx, JS::HandleValue val, size_t *encoded_len);

    template <const char *prefix, uint8_t prefix_len>
    static bool console_out(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        JS::CallArgs args = CallArgsFromVp(argc, vp);
        size_t msg_len;
        JS::UniqueChars msg = encode_log(cx, args.get(0), &msg_len);
        if (!msg)
            return false;

        // printf("%s: %s\n", prefix, msg.get());
        printf("%s\n", msg.get());

        fflush(stdout);

        args.rval().setUndefined();
        return true;
    }

    static constexpr char PREFIX_LOG[] = "Log";
    static constexpr char PREFIX_TRACE[] = "Trace";
    static constexpr char PREFIX_INFO[] = "Info";
    static constexpr char PREFIX_WARN[] = "Warn";
    static constexpr char PREFIX_ERROR[] = "Error";

    const JSFunctionSpec methods[] = {
        JS_FN("log", (console_out<PREFIX_LOG, 3>), 1, JSPROP_ENUMERATE),
        JS_FN("trace", (console_out<PREFIX_TRACE, 5>), 1, JSPROP_ENUMERATE),
        JS_FN("info", (console_out<PREFIX_INFO, 4>), 1, JSPROP_ENUMERATE),
        JS_FN("warn", (console_out<PREFIX_WARN, 4>), 1, JSPROP_ENUMERATE),
        JS_FN("error", (console_out<PREFIX_ERROR, 5>), 1, JSPROP_ENUMERATE),
        JS_FS_END};

    static bool create(JSContext *cx, JS::HandleObject global)
    {
        JS::RootedObject console(cx, JS_NewPlainObject(cx));
        if (!console)
            return false;
        if (!JS_DefineProperty(cx, global, "console", console, JSPROP_ENUMERATE))
            return false;
        return JS_DefineFunctions(cx, console, methods);
    }

    JS::UniqueChars encode_log(JSContext *cx, JS::HandleString str, size_t *encoded_len)
    {
        JS::UniqueChars text = JS_EncodeStringToUTF8(cx, str);
        if (!text)
            return nullptr;

        JSLinearString *linear = JS_EnsureLinearString(cx, str);
        *encoded_len = JS::GetDeflatedUTF8StringLength(linear);
        return text;
    }

    JS::UniqueChars encode_log(JSContext *cx, JS::HandleValue val, size_t *encoded_len)
    {
        JS::RootedString str(cx, JS::ToString(cx, val));
        if (!str)
            return nullptr;

        return encode_log(cx, str, encoded_len);
    }
}

static constexpr const JSClassOps class_ops = {};
static const uint32_t class_flags = 0;

#define CLASS_BOILERPLATE_CUSTOM_INIT(cls)                                                           \
    const JSClass class_ = {#cls, JSCLASS_HAS_RESERVED_SLOTS(Slots::Count) | class_flags,            \
                            &class_ops};                                                             \
    static JS::PersistentRooted<JSObject *> proto_obj;                                               \
                                                                                                     \
    bool is_instance(JSObject *obj)                                                                  \
    {                                                                                                \
        return JS::GetClass(obj) == &class_;                                                         \
    }                                                                                                \
                                                                                                     \
    bool is_instance(JS::Value val)                                                                  \
    {                                                                                                \
        return val.isObject() && is_instance(&val.toObject());                                       \
    }                                                                                                \
                                                                                                     \
    bool check_receiver(JSContext *cx, JS::HandleObject self, const char *method_name)               \
    {                                                                                                \
        if (!is_instance(self))                                                                      \
        {                                                                                            \
            JS_ReportErrorUTF8(cx, "Method %s called on receiver that's not an instance of %s\n",    \
                               method_name, class_.name);                                            \
            return false;                                                                            \
        }                                                                                            \
        return true;                                                                                 \
    };                                                                                               \
                                                                                                     \
    bool init_class_impl(JSContext *cx, JS::HandleObject global,                                     \
                         JS::HandleObject parent_proto = nullptr)                                    \
    {                                                                                                \
        proto_obj.init(cx, JS_InitClass(cx, global, parent_proto, &class_, constructor, ctor_length, \
                                        properties, methods, nullptr, nullptr));                     \
        return proto_obj;                                                                            \
    };

#define CLASS_BOILERPLATE(cls)                              \
    CLASS_BOILERPLATE_CUSTOM_INIT(cls)                      \
                                                            \
    bool init_class(JSContext *cx, JS::HandleObject global) \
    {                                                       \
        return init_class_impl(cx, global);                 \
    }

#define CLASS_BOILERPLATE_NO_CTOR(cls)                                             \
    bool constructor(JSContext *cx, unsigned argc, JS::Value *vp)                  \
    {                                                                              \
        JS_ReportErrorUTF8(cx, #cls " can't be instantiated directly");            \
        return false;                                                              \
    }                                                                              \
                                                                                   \
    CLASS_BOILERPLATE_CUSTOM_INIT(cls)                                             \
                                                                                   \
    bool init_class(JSContext *cx, JS::HandleObject global)                        \
    {                                                                              \
        /* Right now, deleting the ctor from the global object after class         \
           initialization seems to be the best we can do. Not ideal, but works. */ \
        return init_class_impl(cx, global) &&                                      \
               JS_DeleteProperty(cx, global, class_.name);                         \
    }

#define METHOD_HEADER(required_argc)                       \
    /*                                                     \
    // printf("method: %s\n", __func__);                   \
    */                                                     \
    JS::CallArgs args = JS::CallArgsFromVp(argc, vp);      \
    if (!args.requireAtLeast(cx, __func__, required_argc)) \
        return false;                                      \
    JS::RootedObject self(cx, &args.thisv().toObject());   \
    if (!check_receiver(cx, self, __func__))               \
        return false;

namespace TextDecoder
{
    namespace Slots
    {
        enum
        {
            Count
        };
    };

    JSObject *create(JSContext *cx);

    bool constructor(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        JS::CallArgs args = CallArgsFromVp(argc, vp);

        JS::RootedObject self(cx, create(cx));
        if (!self)
            return false;

        args.rval().setObject(*self);
        return true;
    }

    const unsigned ctor_length = 0;

    bool check_receiver(JSContext *cx, JS::HandleObject self, const char *method_name);

    bool decode(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        METHOD_HEADER(1)

        // Default to empty string if no input is given.
        if (args.get(0).isUndefined())
        {
            args.rval().set(JS_GetEmptyStringValue(cx));
            return true;
        }

        if (!args[0].isObject() || !(JS_IsArrayBufferViewObject(&args[0].toObject()) ||
                                     JS::IsArrayBufferObject(&args[0].toObject())))
        {
            JS_ReportErrorUTF8(cx, "TextDecoder#decode: input must be of type ArrayBuffer or ArrayBufferView");
            return false;
        }

        JS::RootedObject input(cx, &args[0].toObject());
        size_t length;
        uint8_t *data;
        bool is_shared;

        if (JS_IsArrayBufferViewObject(input))
        {
            js::GetArrayBufferViewLengthAndData(input, &length, &is_shared, &data);
        }
        else
        {
            JS::GetArrayBufferLengthAndData(input, &length, &is_shared, &data);
        }

        JS::RootedString str(cx, JS_NewStringCopyUTF8N(cx, JS::UTF8Chars((char *)data, length)));
        if (!str)
            return false;

        args.rval().setString(str);
        return true;
    }

    bool encoding_get(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        METHOD_HEADER(0)

        JS::RootedString str(cx, JS_NewStringCopyN(cx, "utf-8", 5));
        if (!str)
            return false;

        args.rval().setString(str);
        return true;
    }

    const JSFunctionSpec methods[] = {
        JS_FN("decode", decode, 1, 0),
        JS_FS_END};

    const JSPropertySpec properties[] = {
        JS_PSG("encoding", encoding_get, 0),
        JS_PS_END};

    CLASS_BOILERPLATE(TextDecoder)

    JSObject *create(JSContext *cx)
    {
        return JS_NewObjectWithGivenProto(cx, &class_, proto_obj);
    }
}

namespace TextEncoder
{
    namespace Slots
    {
        enum
        {
            Count
        };
    };

    JSObject *create(JSContext *cx);

    bool constructor(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        JS::CallArgs args = JS::CallArgsFromVp(argc, vp);

        JS::RootedObject self(cx, create(cx));
        if (!self)
            return false;

        args.rval().setObject(*self);
        return true;
    }

    const unsigned ctor_length = 0;

    bool check_receiver(JSContext *cx, JS::HandleObject self, const char *method_name);

    bool encode(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        METHOD_HEADER(1)

        // Default to empty string if no input is given.
        if (args.get(0).isUndefined())
        {
            JS::RootedObject byte_array(cx, JS_NewUint8Array(cx, 0));
            if (!byte_array)
                return false;

            args.rval().setObject(*byte_array);
            return true;
        }

        size_t chars_len;
        JS::UniqueChars chars = Console::encode_log(cx, args[0], &chars_len);

        auto *rawChars = chars.release();
        JS::RootedObject buffer(cx, JS::NewArrayBufferWithContents(cx, chars_len, rawChars));
        if (!buffer)
        {
            JS_free(cx, rawChars);
            return false;
        }

        JS::RootedObject byte_array(cx, JS_NewUint8ArrayWithBuffer(cx, buffer, 0, chars_len));
        if (!byte_array)
            return false;

        args.rval().setObject(*byte_array);
        return true;
    }

    bool encoding_get(JSContext *cx, unsigned argc, JS::Value *vp)
    {
        METHOD_HEADER(0)

        JS::RootedString str(cx, JS_NewStringCopyN(cx, "utf-8", 5));
        if (!str)
            return false;

        args.rval().setString(str);
        return true;
    }

    const JSFunctionSpec methods[] = {
        JS_FN("encode", encode, 1, 0),
        JS_FS_END};

    const JSPropertySpec properties[] = {
        JS_PSG("encoding", encoding_get, 0),
        JS_PS_END};

    CLASS_BOILERPLATE(TextEncoder)

    JSObject *create(JSContext *cx)
    {
        return JS_NewObjectWithGivenProto(cx, &class_, proto_obj);
    }
}
