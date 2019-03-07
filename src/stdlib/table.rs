use gc_arena::MutationContext;
use gc_sequence as sequence;

use crate::{Callback, CallbackResult, Error, Root, RuntimeError, String, Table, Value};

macro_rules! runtime_err {
    ($mc:expr, $($arg:expr),+) => {
        Error::RuntimeError(RuntimeError(Value::String(String::new($mc, format!($($arg),+).as_bytes()))))
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

    table
        .set(
            mc,
            String::new_static(b"concat"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let list = get_table_arg(args.get(0), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #1 to concat (table expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let sep: Value<'gc> = match args.get(1).cloned().unwrap_or(Value::Nil) {
                        s @ Value::String(_) => s,
                        Value::Nil => Value::String(String::new_static(b"")),
                        v => {
                            return Err(runtime_err!(
                                mc,
                                "bad argument #2 to concat (string expected, got {})",
                                v.type_name()
                            ))
                        }
                    };
                    let start = get_number_arg(args.get(2), Some(1)).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #3 to concat (number expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let end = get_number_arg(args.get(3), Some(list.length())).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #4 to concat (number expected, got {})",
                            v.type_name()
                        )
                    })?;

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

    table
        .set(
            mc,
            String::new_static(b"move"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let src = get_table_arg(args.get(0), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #1 to move (table expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let src_start = get_number_arg(args.get(1), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #2 to move (number expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let src_end = get_number_arg(args.get(2), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #3 to move (number expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let dest_start = get_number_arg(args.get(3), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #4 to move (number expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let dest = get_table_arg(args.get(0), Some(src.clone())).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #5 to move (table expected, got {})",
                            v.type_name()
                        )
                    })?;

                    for i in src_start..=src_end {
                        dest.set(mc, dest_start + (i - src_start), src.get(i))?;
                    }

                    Ok(CallbackResult::Return(vec![Value::Table(dest)]))
                }))
            }),
        )
        .unwrap();

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

    table
        .set(
            mc,
            String::new_static(b"unpack"),
            Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
                Ok(sequence::from_fn_with(args, |mc, args| {
                    let list = get_table_arg(args.get(0), None).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #1 to move (table expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let start = get_number_arg(args.get(1), Some(1)).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #2 to unpack (number expected, got {})",
                            v.type_name()
                        )
                    })?;
                    let end = get_number_arg(args.get(2), Some(list.length())).map_err(|v| {
                        runtime_err!(
                            mc,
                            "bad argument #3 to unpack (number expected, got {})",
                            v.type_name()
                        )
                    })?;

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
