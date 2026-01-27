use super::Interpreter;
use super::values::RuntimeVal;
use crate::grammar::grammar::{self, Expression, Statement};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

impl Interpreter {
    pub fn eval_expr(&mut self, expr: Expression) -> Result<RuntimeVal, String> {
        match expr {
            Expression::StructInit(name, _, fields, _) => {
                // 1. Evaluate all fields
                let mut data = HashMap::new();
                for f in fields {
                    let val = self.eval_expr(f.value)?;
                    data.insert(f.name.value, val);
                }
                // 2. Return Struct Value
                Ok(RuntimeVal::Struct(name.value, data))
            }

            Expression::FieldAccess(target, _, field) => {
                let val = self.eval_expr(*target)?;

                // AUTO-DEREF LOGIC
                // Check if it's a struct directly OR a pointer to a struct
                match val {
                    RuntimeVal::Struct(_, fields) => fields
                        .get(&field.value)
                        .cloned()
                        .ok_or_else(|| format!("Field '{}' not found", field.value)),
                    // Handle Pointer to Struct (Auto-Deref) could go here
                    _ => Err(format!(
                        "Cannot access field '{}' on this type",
                        field.value
                    )),
                }
            }

            Expression::Variable(v) => {
                self.env
                    .get(&v.value) // Use .value here
                    .map(|val| val.data.clone())
                    .ok_or_else(|| format!("ERROR: Variable '{}' not found.", v.value))
            }

            Expression::Number(num_val) => {
                let n: f64 = num_val.value.parse().map_err(|_| "Invalid number")?;
                Ok(RuntimeVal::Float(n))
            }

            // FIXED: Unwrap StringVal and strip quotes
            Expression::StringLit(s) => {
                let content = &s.value[1..s.value.len() - 1];
                Ok(RuntimeVal::String(content.to_string()))
            }
            Expression::BoolLit(b) => match b {
                grammar::BoolVal::True(_) => Ok(RuntimeVal::Bool(true)),
                grammar::BoolVal::False(_) => Ok(RuntimeVal::Bool(false)),
            },
            // 3. Pipe Init
            Expression::PipeInit(_, _) => {
                let (tx, rx) = mpsc::channel();
                Ok(RuntimeVal::Pipe(tx, Arc::new(Mutex::new(rx))))
            }

            // 4. Take (Sync Receive)
            Expression::Take(_, channel_expr) => {
                let chan = self.eval_expr(*channel_expr)?;

                if let RuntimeVal::Pipe(_, rx_mutex) = chan {
                    let rx = rx_mutex.lock().unwrap();
                    let val = rx
                        .recv()
                        .map_err(|_| "Pipe Error: Channel empty or closed".to_string())?;
                    Ok(RuntimeVal::Float(val))
                } else {
                    Err("Runtime Error: 'take' expects a pipe.".to_string())
                }
            }
            // Pointer Logic (Interpreter Stub)
            // Implementing true shared memory in a tree-walker is hard.
            // For now, we just pass the value through (Copy semantics) to fix the build.
            Expression::Ref(_, target) => {
                let val = self.eval_expr(*target)?;
                Ok(val) // "Fake" reference
            }
            Expression::Deref(_, target) => {
                let val = self.eval_expr(*target)?;
                Ok(val) // "Fake" dereference
            }

            // 2. List Init
            Expression::ListInit(_, _, _, items, _) => {
                let mut vec = Vec::new();
                for i in items {
                    vec.push(self.eval_expr(i)?);
                }
                Ok(RuntimeVal::List(vec))
            }

            // 3. Map Init
            Expression::MapInit(_, _, _, _, pairs, _) => {
                let mut map = HashMap::new();
                for p in pairs {
                    let k = self.eval_expr(p.key)?.to_string();
                    let v = self.eval_expr(p.value)?;
                    map.insert(k, v);
                }
                Ok(RuntimeVal::Map(map))
            }

            // 4. AT Command
            Expression::At(col, _, key_expr) => {
                let collection = self.eval_expr(*col)?;
                let key = self.eval_expr(*key_expr)?;

                match collection {
                    RuntimeVal::List(vec) => {
                        let idx = key.as_float()? as usize;
                        vec.get(idx)
                            .cloned()
                            .ok_or_else(|| "Index out of bounds".to_string())
                    }
                    RuntimeVal::Map(map) => {
                        let k_str = key.to_string();
                        map.get(&k_str)
                            .cloned()
                            .ok_or_else(|| "Key not found".to_string())
                    }
                    _ => Err("Cannot use 'at' on this type".to_string()),
                }
            }

            // 5. PUSH Command (Interpreter Warning)
            Expression::Push(col_expr, _, val_expr) => {
                println!("⚠️ Interpreter: 'push' ignored (compile to Rust for mutation).");
                let _ = self.eval_expr(*col_expr)?;
                let _ = self.eval_expr(*val_expr)?;
                Ok(RuntimeVal::Void)
            }
            Expression::Range(start, _, end) => {
                let s = self.eval_expr(*start)?.as_float()? as i64;
                let e = self.eval_expr(*end)?.as_float()? as i64;
                Ok(RuntimeVal::Range(s, e))
            }
            Expression::Add(lhs, _, rhs) => {
                let l = self.eval_expr(*lhs)?;
                let r = self.eval_expr(*rhs)?;
                match (l, r) {
                    (RuntimeVal::Float(a), RuntimeVal::Float(b)) => Ok(RuntimeVal::Float(a + b)),
                    (RuntimeVal::String(a), RuntimeVal::String(b)) => {
                        Ok(RuntimeVal::String(format!("{}{}", a, b)))
                    }
                    _ => Err("Runtime Error: Can only ADD numbers or strings".to_string()),
                }
            }
            Expression::Len(_, expr) => match self.eval_expr(*expr)? {
                RuntimeVal::String(s) => Ok(RuntimeVal::Float(s.len() as f64)),
                RuntimeVal::List(l) => Ok(RuntimeVal::Float(l.len() as f64)),
                RuntimeVal::Map(m) => Ok(RuntimeVal::Float(m.len() as f64)),
                _ => Err("Runtime Error: 'len' only supports string, list, map.".to_string()),
            },
            Expression::Sub(lhs, _, rhs) => {
                let l = self.eval_expr(*lhs)?;
                let r = self.eval_expr(*rhs)?;
                match (l, r) {
                    (RuntimeVal::Float(a), RuntimeVal::Float(b)) => Ok(RuntimeVal::Float(a - b)),
                    _ => Err("Runtime Error: Can only SUBTRACT numbers".to_string()),
                }
            }
            Expression::Mul(lhs, _, rhs) => {
                let l = self.eval_expr(*lhs)?;
                let r = self.eval_expr(*rhs)?;
                match (l, r) {
                    (RuntimeVal::Float(a), RuntimeVal::Float(b)) => Ok(RuntimeVal::Float(a * b)),
                    _ => Err("Runtime Error: Can only MULTIPLY numbers".to_string()),
                }
            }
            Expression::Div(lhs, _, rhs) => {
                let l = self.eval_expr(*lhs)?;
                let r = self.eval_expr(*rhs)?;
                match (l, r) {
                    (RuntimeVal::Float(a), RuntimeVal::Float(b)) => Ok(RuntimeVal::Float(a / b)),
                    _ => Err("Runtime Error: Can only DIVIDE numbers".to_string()),
                }
            }
            Expression::Gt(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? > self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            Expression::Lt(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? < self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            Expression::Eq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? == self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            Expression::Neq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? != self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            Expression::Geq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? >= self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            Expression::Leq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? <= self.eval_expr(*rhs)? {
                    1.0
                } else {
                    0.0
                };
                Ok(RuntimeVal::Float(val))
            }
            // 1. Handle Standard Calls
            Expression::Call(func_var, _, args, _) => {
                // A. Resolve the function name
                let func_name = match *func_var {
                    Expression::Variable(v) => v.value,
                    _ => return Err("Expected function name".to_string()),
                };

                // B. Retrieve the function code from our storage
                // We verify it exists and is actually a FunctionDef
                let func_stmt = self
                    .functions
                    .get(&func_name)
                    .cloned() // Clone it so we don't fight the borrow checker
                    .ok_or_else(|| format!("Undefined function: '{}'", func_name))?;

                if let Statement::FunctionDef {
                    params,
                    body,
                    pure_kw: _,
                    ..
                } = func_stmt
                {
                    // C. Purity Check (The "Sandbox")
                    // If pure_kw is Some, we should enable strict mode (TODO for next step)

                    // D. Evaluate Arguments *in the current scope*
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg)?);
                    }

                    // E. Create the "Stack Frame" (Local Scope)
                    // We save the old environment to restore it later
                    let old_env = self.env.clone();

                    // We create a fresh environment.
                    // Note: For true "Lexical Scoping", we should copy global variables in,
                    // but for "Pure" functions, we might want an empty map!
                    // For now, let's clone the global scope so we can read globals.
                    let mut fn_env = self.env.clone();

                    // F. Bind Arguments to Parameters
                    if params.len() != arg_values.len() {
                        return Err(format!(
                            "Function '{}' expects {} args, got {}.",
                            func_name,
                            params.len(),
                            arg_values.len()
                        ));
                    }

                    for (i, param) in params.into_iter().enumerate() {
                        fn_env.insert(
                            param.name,
                            super::values::Value {
                                data: arg_values[i].clone(),
                                is_mutable: true, // Args are local variables, so they are mutable
                            },
                        );
                    }

                    // G. Context Switch!
                    self.env = fn_env;

                    // H. Run the Body
                    let result_sig = self.execute_block(body)?;

                    // I. Restore the Old World
                    self.env = old_env;

                    // Return the result of the function
                    match result_sig {
                        super::StmtResult::Normal(v) => Ok(v),
                        super::StmtResult::Return(v) => Ok(v),
                        super::StmtResult::Break | super::StmtResult::Continue => {
                            Err("Error: 'break' or 'continue' leaked from function body."
                                .to_string())
                        }
                    }
                } else {
                    Err(format!("'{}' is not a function.", func_name))
                }
            }

            // 2. Handle 'Run' Calls
            // For the Interpreter (Test Bench), we run this Synchronously.
            // Why? Because implementing true threading in a tree-walker is overkill.
            // The Compiler will handle the real async/parallelism.
            Expression::RunCall(_, call_expr) => {
                println!("⚠️ [Interpreter] Note: 'run' executed synchronously in test mode.");
                self.eval_expr(*call_expr)
            }
        }
    }
}
