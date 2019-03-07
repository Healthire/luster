use gc_arena::MutationContext;
use gc_sequence as sequence;

use crate::{Callback, CallbackResult, Error, Root, RuntimeError, String, Table, Value};

macro_rules! runtime_err {
    ($mc:expr, $($arg:expr),+) => {
        Error::RuntimeError(RuntimeError(Value::String(String::new($mc, format!($($arg),+).as_bytes()))))
    };
}

macro_rules! bad_arg {
    ($mc:expr, $f:expr, $i:expr, $e:expr, $v:expr) => {
        Error::RuntimeError(RuntimeError(Value::String(String::new(
            $mc,
            format!(
                "bad argument #{} to {} ({} expected, got {})",
                $i,
                $f,
                $e,
                $v.type_name()
            )
            .as_bytes(),
        ))))
    };
}

pub fn load_table<'gc>(mc: MutationContext<'gc, '_>, _: Root<'gc>, env: Table<'gc>) {
    let table = Table::new(mc);

    fn get_table_arg<'gc, 'a>(
        v: Option<&'a Value<'gc>>,
        def: Option<Table<'gc>>,
    ) -> Result<Table<'gc>, &'a Value<'gc>> {
        match (v.unwrap_or(&Value::Nil), def) {
            (Value::Table(t), _) => Ok(t.clone()),
            (Value::Nil, Some(t)) => Ok(t),
            (v, _) => Err(v),
        }
    }

    fn get_number_arg<'gc, 'a>(
        v: Option<&'a Value<'gc>>,
        def: Option<i64>,
    ) -> Result<i64, &'a Value<'gc>> {
        match (v.unwrap_or(&Value::Nil), def) {
            (Value::Integer(i), _) => Ok(*i),
            (Value::Number(i), _) => Ok(i.floor() as i64),
            (Value::Nil, Some(def)) => Ok(def),
            (v, _) => Err(v),
        }
    }

    // concat
    table
        .set(
            mc,
            String::new_static(b"concat"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let list = get_table_arg(args.get(0), None)
                        .map_err(|v| bad_arg!(mc, "concat", 1, "table", v))?;
                    let sep: Value<'gc> = match args.get(1).cloned().unwrap_or(Value::Nil) {
                        s @ Value::String(_) => s,
                        Value::Nil => Value::String(String::new_static(b"")),
                        v => return Err(bad_arg!(mc, "concat", 2, "string", v)),
                    };
                    let start = get_number_arg(args.get(2), Some(1))
                        .map_err(|v| bad_arg!(mc, "concat", 3, "number", v))?;
                    let end = get_number_arg(args.get(3), Some(list.length()))
                        .map_err(|v| bad_arg!(mc, "concat", 4, "number", v))?;

                    let mut strings = Vec::new();
                    for i in start..=end {
                        match list.get(i) {
                            v @ Value::String(_) => {
                                strings.push(v);
                                if i < end {
                                    strings.push(sep);
                                }
                            }
                            v => {
                                return Err(runtime_err!(
                                    mc,
                                    "invalid value ({}) at index {} in table for 'concat'",
                                    v.type_name(),
                                    i
                                ));
                            }
                        }
                    }

                    Ok(CallbackResult::Return(vec![Value::String(String::concat(
                        mc, &strings,
                    )?)]))
                }))
            }),
        )
        .unwrap();

    // move
    table
        .set(
            mc,
            String::new_static(b"move"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let src = get_table_arg(args.get(0), None)
                        .map_err(|v| bad_arg!(mc, "move", 1, "table", v))?;
                    let src_start = get_number_arg(args.get(1), None)
                        .map_err(|v| bad_arg!(mc, "move", 2, "number", v))?;
                    let src_end = get_number_arg(args.get(2), None)
                        .map_err(|v| bad_arg!(mc, "move", 3, "number", v))?;
                    let dest_start = get_number_arg(args.get(3), None)
                        .map_err(|v| bad_arg!(mc, "move", 4, "number", v))?;
                    let dest = get_table_arg(args.get(0), Some(src.clone()))
                        .map_err(|v| bad_arg!(mc, "move", 5, "table", v))?;

                    for i in src_start..=src_end {
                        dest.set(mc, dest_start + (i - src_start), src.get(i))?;
                    }

                    Ok(CallbackResult::Return(vec![Value::Table(dest)]))
                }))
            }),
        )
        .unwrap();

    // pack
    table
        .set(
            mc,
            String::new_static(b"pack"),
            Callback::new_sequence(mc, |args| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let t = Table::new(mc);
                    for (i, v) in args.into_iter().enumerate() {
                        t.set(mc, i as i64 + 1, v)?;
                    }

                    Ok(CallbackResult::Return(vec![Value::Table(t)]))
                }))
            }),
        )
        .unwrap();

    // unpack
    table
        .set(
            mc,
            String::new_static(b"unpack"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let list = get_table_arg(args.get(0), None)
                        .map_err(|v| bad_arg!(mc, "unpack", 1, "table", v))?;
                    let start = get_number_arg(args.get(1), Some(1))
                        .map_err(|v| bad_arg!(mc, "unpack", 2, "number", v))?;
                    let end = get_number_arg(args.get(2), Some(list.length()))
                        .map_err(|v| bad_arg!(mc, "unpack", 3, "number", v))?;

                    let mut unpacked = Vec::new();
                    for i in start..=end {
                        unpacked.push(list.get(i));
                    }

                    Ok(CallbackResult::Return(unpacked))
                }))
            }),
        )
        .unwrap();

    env.set(mc, String::new_static(b"table"), table).unwrap();
}
