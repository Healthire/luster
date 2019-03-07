use gc_arena::MutationContext;
use gc_sequence as sequence;

use crate::{Callback, CallbackResult, Root, RuntimeError, String, Table, Value};

macro_rules! runtime_err {
    ($mc:expr, $($arg:expr),+) => {
        RuntimeError(Value::String(String::new($mc, format!($($arg),+).as_bytes()))).into()
    };
}

pub fn load_table<'gc>(mc: MutationContext<'gc, '_>, _: Root<'gc>, env: Table<'gc>) {
    let table = Table::new(mc);

    table.set(
        mc,
        String::new_static(b"concat"),
        Callback::new_sequence(mc, |args: Vec<Value<'gc>>| {
            Ok(sequence::from_fn_with(args, |mc, args| {
                let list = match args.get(0).unwrap_or(&Value::Nil) {
                    Value::Table(t) => t,
                    v => {
                        return Err(runtime_err!(
                            mc,
                            "bad argument #1 to concat (table expected, got {})",
                            v.type_name()
                        ))
                    }
                };
                
                let sep: Value<'gc> = match args
                    .get(1).cloned()
                    .unwrap_or(Value::Nil)
                {
                    s @ Value::String(_) => s,
                    Value::Nil => Value::String(String::new_static(b"")),
                    v => {
                        return Err(runtime_err!(
                            mc,
                            "bad argument #2 to concat (table expected, got {})",
                            v.type_name()
                        ))
                    }
                };
                let start = match args.get(2).unwrap_or(&Value::Nil) {
                    Value::Integer(i) => *i,
                    Value::Number(i) => i.floor() as i64,
                    Value::Nil => 1,
                    v => {
                        return Err(runtime_err!(
                            mc,
                            "bad argument #3 to concat (number expected, got {})",
                            v.type_name()
                        ))
                    }
                };
                let end = match args.get(3).unwrap_or(&Value::Nil) {
                    Value::Integer(i) => *i,
                    Value::Number(i) => i.floor() as i64,
                    Value::Nil => list.length(),
                    v => {
                        return Err(runtime_err!(
                            mc,
                            "bad argument #4 to concat (number expected, got {})",
                            v.type_name()
                        ))
                    }
                };
                
                let mut strings = Vec::new();
                for i in start..=end {
                    match list.get(i) {
                        v @ Value::String(_) => {
                            strings.push(v);
                            if i < end {
                                strings.push(sep);
                            }
                        },
                        v => {return Err(runtime_err!(
                            mc,
                            "invalid value ({}) at index {} in table for 'concat'",
                            v.type_name(),
                            i
                        ));},
                    }
                }

                Ok(CallbackResult::Return(vec![Value::String(String::concat(mc, &strings)?)]))
            })
        }),
    ).unwrap();

    env.set(mc, String::new_static(b"table"), table).unwrap();
}
