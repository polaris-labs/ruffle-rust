use crate::avm1::{ScriptObject, Avm1, Value, Error, TObject};
use gc_arena::MutationContext;
use crate::avm1::function::{Executable, FunctionObject};
use enumset::EnumSet;
use crate::context::UpdateContext;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::object::Object;
use json::JsonValue;


pub fn delete_all<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.deleteAll() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_disk_usage<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getDiskUsage() not implemented");
    Ok(Value::Undefined.into())
}

fn parse_json<'gc>(json_obj: JsonValue, avm: &mut Avm1<'gc>, object: Object<'gc>, context: &mut UpdateContext<'_, 'gc, '_>) {
    for entry in json_obj.entries() {
        match entry.1 {
            JsonValue::Null => {
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Null,
                    EnumSet::empty()
                );
            },
            JsonValue::Short(s) => {
                let val: String = s.as_str().to_string();
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::String(val),
                    EnumSet::empty()
                );
            },
            JsonValue::String(s) => {
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::String(s.clone()),
                    EnumSet::empty()
                );
            },
            JsonValue::Number(f) => {
                log::warn!("[SharedObject] {} = {}", entry.0, f);
                let val: f64 = f.clone().into();
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Number(val),
                    EnumSet::empty()
                );
            },
            JsonValue::Boolean(b) => {
                log::warn!("[SharedObject] {} = {}", entry.0, b);
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Bool(*b),
                    EnumSet::empty()
                );
            },
            JsonValue::Object(o) => {
                let so = avm.prototypes().object;
                let obj = so.new(avm, context, so, &[]).unwrap();
                let _ = crate::avm1::globals::object::constructor(avm, context, obj, &[]).unwrap();
                parse_json(JsonValue::Object(o.clone()),  avm,obj, context);

                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Object(obj),
                    EnumSet::empty()
                );
            },
            JsonValue::Array(_) => {},
        }
    }
}

pub fn get_local<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {

    //TODO: use args and restore existing state (after disk backend)

    let so = avm.prototypes().shared_object;
    let obj = so.new(avm, action_context, so, &[])?;
    let _ = constructor(avm, action_context, obj, &[])?;

    let saved = action_context.storage.get_string("tmp".to_string());
    if let Some(saved_data) = saved {
        let data = obj.get("data", avm, action_context).unwrap().as_object().unwrap();
        //TODO: error handle
        let js = json::parse(&saved_data).unwrap();
        parse_json(js, avm, data, action_context);
    }

    // log::warn!("SharedObject.getLocal() not implemented");
    Ok(obj.into())
}

pub fn get_remote<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getRemote() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_max_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getMaxSize() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_shared_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let shared_obj = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        array_proto,
    );
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function(
        "deleteAll",
        delete_all,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getDiskUsage",
        get_disk_usage,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getLocal",
        get_local,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getRemote",
        get_remote,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getMaxSize",
        get_max_size,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );
    // TODO: listeners, might not have _listeners prop

    shared_obj
}

pub fn clear<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {

    //TODO: does this remove elements not in data prop (e.g. recreate entire obj) or does it just remove children of prop

    let data = this.get("data", avm, action_context).unwrap().as_object().unwrap();

    for k in &data.get_keys(avm) {
        data.delete(avm, action_context.gc_context, k);
    }

    Ok(Value::Undefined.into())
}

pub fn close<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.close() not implemented");
    Ok(Value::Undefined.into())
}

pub fn connect<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.connect() not implemented");
    Ok(Value::Undefined.into())
}

fn recursive_serialize<'gc>(avm: &mut Avm1<'gc>, action_context: &mut UpdateContext<'_, 'gc, '_>, obj: Object<'gc>, json_obj: &mut JsonValue) {
    for k in &obj.get_keys(avm) {
        let elem = obj.get(k, avm, action_context).unwrap();

        match elem {
            //TODO: should never happen
            Value::Undefined => log::warn!(" [SharedObject] {} = Undef", k),
            Value::Null => json_obj[k] = JsonValue::Null,
            Value::Bool(b) => json_obj[k] = b.into(),
            Value::Number(f) => json_obj[k] = f.into(),
            Value::String(s) => json_obj[k] = s.into(),
            Value::Object(o) => {
                // Don't attempt to serialize functions, etc
                if !o.is_instance_of(avm, action_context, o, avm.prototypes().function).unwrap() {

                    let mut sub_data_json = JsonValue::new_object();
                    recursive_serialize(avm, action_context, o, &mut sub_data_json);
                    json_obj[k] = sub_data_json;
                }
            },
        }
    }
}

pub fn flush<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //TODO: consider args
    let data = _this.get("data", _avm, _action_context).unwrap().as_object().unwrap();
    let mut data_json = JsonValue::new_object();

    recursive_serialize(_avm, _action_context,data, &mut data_json);

    //TODO: somehow need to know the name of where to save it to (hidden property?)
    _action_context.storage.put_string("tmp".into(), data_json.dump());
    //TODO: return value
    Ok(Value::Undefined.into())
}

pub fn get_size<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {

    let name = args.get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(avm, action_context)?;

    // TODO: what does this return if the item dosent exist
    let size = action_context.storage.get_size(name).unwrap_or(0) as f64;

    Ok(size.into())
}

pub fn send<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.send() not implemented");
    Ok(Value::Undefined.into())
}

pub fn set_fps<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.setFps() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onStatus() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_sync<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onSync() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj = ScriptObject::object(gc_context, Some(proto));
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function(
        "clear",
        clear,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "close",
        close,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "connect",
        connect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "flush",
        flush,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "getSize",
        get_size,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "send",
        send,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "setFps",
        set_fps,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onSync",
        on_sync,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    shared_obj.into()
}

pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {


    let so = avm.prototypes().object;
    let obj = so.new(avm, context, so, &[])?;
    let _ = crate::avm1::globals::object::constructor(avm, context, obj, &[])?;

    //TODO: verify args, esp. this
    this.define_value(
        context.gc_context,
        "data",
        obj.into(),
        EnumSet::empty(),
    );

    Ok(Value::Undefined.into())
}
