use crate::grammar::{self, Expression, Statement};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum RuntimeVal {
    Float(f64),
    String(String),
    Bool(bool),
    Range(i64, i64),
    Void,
    Pipe(Sender<f64>, Arc<Mutex<Receiver<f64>>>),
    Struct(String, HashMap<String, RuntimeVal>),
    List(Vec<RuntimeVal>),
    Map(HashMap<String, RuntimeVal>),
}

// Manual implementation to handle Pipe which cannot be compared
impl PartialEq for RuntimeVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeVal::Float(a), RuntimeVal::Float(b)) => a == b,
            (RuntimeVal::String(a), RuntimeVal::String(b)) => a == b,
            (RuntimeVal::Bool(a), RuntimeVal::Bool(b)) => a == b,
            (RuntimeVal::Range(s1, e1), RuntimeVal::Range(s2, e2)) => s1 == s2 && e1 == e2,
            (RuntimeVal::Void, RuntimeVal::Void) => true,
            // Pipes are never equal (identity check is hard without ID)
            (RuntimeVal::Pipe(_, _), RuntimeVal::Pipe(_, _)) => false,
            // Structs equality
            (RuntimeVal::Struct(n1, d1), RuntimeVal::Struct(n2, d2)) => n1 == n2 && d1 == d2,
            // Collections equality
            (RuntimeVal::List(l1), RuntimeVal::List(l2)) => l1 == l2,
            (RuntimeVal::Map(m1), RuntimeVal::Map(m2)) => m1 == m2,
            _ => false,
        }
    }
}

impl PartialOrd for RuntimeVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RuntimeVal::Float(a), RuntimeVal::Float(b)) => a.partial_cmp(b),
            (RuntimeVal::String(a), RuntimeVal::String(b)) => a.partial_cmp(b),
            // Other types: define an arbitrary order or return None
            _ => None,
        }
    }
}
impl RuntimeVal {
    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            RuntimeVal::Float(f) => Ok(*f),
            _ => Err("Type Error: Expected a number".to_string()),
        }
    }
}

impl std::fmt::Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeVal::Float(n) => write!(f, "{}", n),
            RuntimeVal::String(s) => write!(f, "{}", s),
            RuntimeVal::Bool(b) => write!(f, "{}", b),
            RuntimeVal::Range(s, e) => write!(f, "{}..{}", s, e),
            RuntimeVal::Void => write!(f, "void"),
            RuntimeVal::Pipe(_, _) => write!(f, "<Pipe>"),
            RuntimeVal::Struct(name, _) => write!(f, "<Struct {}>", name),
            RuntimeVal::List(l) => write!(f, "<List len={}>", l.len()),
            RuntimeVal::Map(m) => write!(f, "<Map len={}>", m.len()),
        }
    }
}
#[derive(Clone, Debug)]
struct Value {
    data: RuntimeVal,
    is_mutable: bool,
}
pub struct Interpreter {
    env: HashMap<String, Value>,
    functions: HashMap<String, Statement>,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    pub fn run(&mut self, program: grammar::Program) -> Result<(), String> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
    fn execute_statement(&mut self, statement: Statement) -> Result<RuntimeVal, String> {
        match statement {
            // Struct definitions are just Declarations, no runtime effect in interpreter
            Statement::StructDef { .. } => Ok(RuntimeVal::Void),
            Statement::Assignment {
                var_kw,
                ident,
                value,
                ..
            } => {
                let val = self.eval_expr(value)?;
                let user_wrote_var = var_kw.is_some();
                let existing_info = self.env.get(&ident).map(|v| v.is_mutable);
                match existing_info {
                    // CASE A: The Variable Already Exists
                    Some(is_mutable) => {
                        if user_wrote_var {
                            return Err(format!(
                                "ERROR: '{}' already exists. Don't use 'var' again.",
                                ident
                            ));
                        }
                        if !is_mutable {
                            return Err(format!(
                                "ERROR: '{}' is immutable (read-only). You cannot change it.",
                                ident
                            ));
                        }
                        // It's mutable, so we update it!
                        self.env.insert(
                            ident.clone(),
                            Value {
                                data: val.clone(),
                                is_mutable: true,
                            },
                        );
                        // println!("🔄 Reassigned Mutable: {} = {}", ident, val);
                    }

                    // CASE B: The Variable is New (Your Fix is Here)
                    None => {
                        // If 'var' is present -> Mutable.
                        // If 'var' is MISSING -> Immutable (The Sane Default).
                        self.env.insert(
                            ident.clone(),
                            Value {
                                data: val.clone(),
                                is_mutable: user_wrote_var,
                            },
                        );

                        /*
                        let type_log = if user_wrote_var {
                            "Mutable"
                        } else {
                            "Immutable"
                        };
                        println!("✨ Defined New {}: {} = {}", type_log, ident, val);
                        */
                    }
                }
                Ok(RuntimeVal::Void)
            }
            Statement::On {
                condition,
                body,
                else_clause,
                ..
            } => {
                // 1. Evaluate the condition (returns 1 for True, 0 for False)
                let result = self.eval_expr(condition)?.as_float()?;

                if result != 0.0 {
                    // True: Run the main block
                    self.execute_block(body)?;
                } else {
                    // False: Run the 'off' block (if it exists)
                    if let Some(clause) = else_clause {
                        self.execute_block(clause.body)?;
                    }
                }
                Ok(RuntimeVal::Void)
            }
            Statement::LoopOn {
                condition, body, ..
            } => {
                // While condition evaluates to True (1)
                while self.eval_expr(condition.clone())?.as_float()? != 0.0 {
                    self.execute_block(body.clone())?;
                }
                Ok(RuntimeVal::Void)
            }
            // ⭐ NEW: For Loop (loop x in 0..10 per 2 ...)
            Statement::LoopIter {
                iterator,
                iterable,
                step,
                filter,
                body,
                else_clause,
                ..
            } => {
                let iterable_val = self.eval_expr(iterable)?;

                // Vector of items to iterate over
                let items: Vec<RuntimeVal> = match iterable_val {
                    RuntimeVal::Range(start, end) => {
                        // Determine Step (Default 1)
                        let step_val = if let Some(s) = step {
                            self.eval_expr(s.value)?.as_float()? as i64
                        } else {
                            1
                        };
                        let mut vec = Vec::new();
                        let mut current = start;
                        while current < end {
                            vec.push(RuntimeVal::Float(current as f64));
                            current += step_val;
                        }
                        vec
                    }
                    RuntimeVal::List(list) => list,
                    RuntimeVal::String(s) => {
                        // Iterate over characters as strings
                        s.chars()
                            .map(|c| RuntimeVal::String(c.to_string()))
                            .collect()
                    }
                    _ => {
                        return Err(
                            "Loop Error: Can only loop over ranges, lists, or strings".to_string()
                        );
                    }
                };

                for item in items {
                    // 1. SAVE the current environment (Parent Scope)
                    let parent_env = self.env.clone();

                    // 2. Define iterator in the CURRENT scope
                    self.env.insert(
                        iterator.clone(),
                        Value {
                            data: item, // loop variable is mutable copy
                            is_mutable: false,
                        },
                    );

                    // 3. Handle Filter
                    let run_main = if let Some(f) = &filter {
                        // Check condition
                        self.eval_expr(f.condition.clone())?.as_float()? != 0.0
                    } else {
                        true
                    };

                    // 4. Run Body
                    if run_main {
                        self.execute_block(body.clone())?;
                    } else if let Some(off) = &else_clause {
                        self.execute_block(off.body.clone())?;
                    }

                    // 5. RESTORE Parent Env
                    self.env = parent_env;
                }
                Ok(RuntimeVal::Void)
            }
            Statement::Print(_, expr) => {
                let val = self.eval_expr(expr)?;
                println!("{}", val);
                Ok(RuntimeVal::Void)
            }
            Statement::ExprStmt(expr) => self.eval_expr(expr),
            stmt @ Statement::FunctionDef { .. } => {
                if let Statement::FunctionDef { name, .. } = &stmt {
                    let func_name = name.clone();
                    self.functions.insert(func_name.clone(), stmt);
                    println!("✨ Registered Function: {}", func_name);
                }
                Ok(RuntimeVal::Void)
            }
            // 1. Give (Sync Send)
            Statement::Give(_, channel_expr, value_expr) => {
                let chan = self.eval_expr(channel_expr)?;
                let val = self.eval_expr(value_expr)?.as_float()?;

                if let RuntimeVal::Pipe(tx, _) = chan {
                    // Send value. unwrap() panics if channel closed/broken.
                    tx.send(val)
                        .map_err(|_| "Pipe Error: Receiver closed".to_string())?;
                } else {
                    return Err("Runtime Error: 'give' expects a pipe.".to_string());
                }
                Ok(RuntimeVal::Void)
            }

            // 2. Close (Drop Sender)
            Statement::Close(_, _channel_expr) => {
                // In MPSC, closing happens when Sender is dropped.
                // Since our 'env' holds clones of Sender, explicit close is tricky in interpreter.
                // We'll just do nothing in the interpreter test bench.
                // The Compiler is the source of truth for async behavior.
                println!("⚠️ [Interpreter] 'close' is a no-op in test mode.");
                Ok(RuntimeVal::Void)
            }
        }
    }
    fn execute_block(&mut self, block: grammar::Block) -> Result<RuntimeVal, String> {
        let mut last_val = RuntimeVal::Void;

        for stmt in block.statements {
            // Capture the result of every statement
            last_val = self.execute_statement(stmt)?;
        }
        Ok(last_val) // Return the result of the last statement
    }
    fn eval_expr(&mut self, expr: Expression) -> Result<RuntimeVal, String> {
        match expr {
            Expression::StructInit(name, _, fields, _) => {
                // 1. Evaluate all fields
                let mut data = HashMap::new();
                for f in fields {
                    let val = self.eval_expr(f.value)?;
                    data.insert(f.name.value, val); // FIXED: Unwrap f.name.value
                }
                // 2. Return Struct Value
                Ok(RuntimeVal::Struct(name.value, data)) // FIXED: Unwrap name.value
            }

            Expression::FieldAccess(target, _, field) => {
                let val = self.eval_expr(*target)?;

                // AUTO-DEREF LOGIC
                // Check if it's a struct directly OR a pointer to a struct
                match val {
                    RuntimeVal::Struct(_, fields) => fields
                        .get(&field.value) // FIXED: Unwrap field.value
                        .cloned()
                        .ok_or_else(|| format!("Field '{}' not found", field.value)),
                    // Handle Pointer to Struct (Auto-Deref) could go here
                    _ => Err(format!(
                        "Cannot access field '{}' on this type",
                        field.value
                    )),
                }
            }

            // FIXED: Unwrap VariableVal
            Expression::Variable(v) => {
                self.env
                    .get(&v.value) // Use .value here
                    .map(|val| val.data.clone())
                    .ok_or_else(|| format!("ERROR: Variable '{}' not found.", v.value))
            }

            // FIXED: Unwrap NumberVal
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
                            Value {
                                data: arg_values[i].clone(),
                                is_mutable: true, // Args are local variables, so they are mutable
                            },
                        );
                    }

                    // G. Context Switch!
                    self.env = fn_env;

                    // H. Run the Body
                    let result = self.execute_block(body);

                    // I. Restore the Old World
                    self.env = old_env;

                    // Return the result of the function
                    // Return the result of the function
                    result
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
