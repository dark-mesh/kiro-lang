use crate::grammar::{self, Expression, Statement};
use std::collections::HashSet;

pub struct Compiler {
    known_vars: HashSet<String>,
}
impl Compiler {
    pub fn new() -> Self {
        Self {
            known_vars: HashSet::new(),
        }
    }
    pub fn compile(&mut self, program: grammar::Program) -> String {
        let mut output = String::new();
        output.push_str("#![allow(unused_mut, unused_variables, unused_parens)]\n");
        // 1. Import async_channel
        output.push_str("use async_channel;\n");

        // 2. Define the Pipe Wrapper
        // We use a struct so we can pass a single 'p' variable that holds both ends.
        output.push_str(
            r#"
            #[derive(Clone)]
            struct KiroPipe<T> {
                tx: async_channel::Sender<T>,
                rx: async_channel::Receiver<T>,
            }

            // --- HELPER TRAIT FOR AUTO-DEREF ---
            trait KiroGet {
                type Inner;
                fn kiro_get<R>(&self, f: impl FnOnce(&Self::Inner) -> R) -> R;
            }

            // Pointer (e.g., Arc<Mutex<User>>)
            impl<T> KiroGet for std::sync::Arc<std::sync::Mutex<T>> {
                type Inner = T;
                fn kiro_get<R>(&self, f: impl FnOnce(&T) -> R) -> R {
                    let guard = self.lock().unwrap();
                    f(&*guard)
                }
            }

            // --- KIRO AT TRAIT (Access Command) ---
            trait KiroAt<I, O> { fn kiro_at(&self, index: I) -> O; }

            // List Implementation: list at 0
            impl<T: Clone> KiroAt<f64, T> for Vec<T> {
                fn kiro_at(&self, index: f64) -> T {
                    self.get(index as usize).cloned().expect("Index out of bounds")
                }
            }

            // Map Implementation: map at "key"
            impl<K, V> KiroAt<K, V> for std::collections::HashMap<K, V> 
            where K: std::hash::Hash + Eq + Clone, V: Clone {
                fn kiro_at(&self, key: K) -> V {
                    self.get(&key).cloned().expect("Key not found")
                }
            }

            // --- KIRO ADD (String Concatenation vs Math) ---
            trait KiroAdd<Rhs = Self> { type Output; fn kiro_add(self, rhs: Rhs) -> Self::Output; }
            impl KiroAdd for f64 { type Output = f64; fn kiro_add(self, rhs: f64) -> f64 { self + rhs } }
            impl KiroAdd for String { type Output = String; fn kiro_add(self, rhs: String) -> String { format!("{}{}", self, rhs) } }

            // --- KIRO LEN ---
            trait KiroLen { fn kiro_len(&self) -> f64; }
            impl<T> KiroLen for Vec<T> { fn kiro_len(&self) -> f64 { self.len() as f64 } }
            impl<K, V> KiroLen for std::collections::HashMap<K, V> { fn kiro_len(&self) -> f64 { self.len() as f64 } }
            impl KiroLen for String { fn kiro_len(&self) -> f64 { self.len() as f64 } }

            // --- KIRO ITER (Looping) ---
            trait KiroIter { type Item; type IntoIter: Iterator<Item = Self::Item>; fn kiro_iter(self) -> Self::IntoIter; }
            impl KiroIter for std::ops::Range<i64> { type Item = i64; type IntoIter = std::ops::Range<i64>; fn kiro_iter(self) -> Self::IntoIter { self } }
            impl<T> KiroIter for Vec<T> { type Item = T; type IntoIter = std::vec::IntoIter<T>; fn kiro_iter(self) -> Self::IntoIter { self.into_iter() } }
            impl KiroIter for String { type Item = char; type IntoIter = std::vec::IntoIter<char>; fn kiro_iter(self) -> Self::IntoIter { self.chars().collect::<Vec<_>>().into_iter() } }

            // --- AS KIRO LOOP VAR (Type Correction) ---
            trait AsKiroLoopVar { type Out; fn as_kiro(self) -> Self::Out; }
            impl AsKiroLoopVar for i64 { type Out = f64; fn as_kiro(self) -> f64 { self as f64 } }
            impl AsKiroLoopVar for f64 { type Out = f64; fn as_kiro(self) -> f64 { self } }
            impl AsKiroLoopVar for char { type Out = String; fn as_kiro(self) -> String { self.to_string() } }
            impl AsKiroLoopVar for String { type Out = String; fn as_kiro(self) -> String { self } }

        "#,
        );
        output.push_str("#[tokio::main]\nasync fn main(){\n");
        for statement in program.statements {
            let line = self.compile_statement(statement);
            output.push_str(&format!("\t{}\n", line));
        }
        output.push_str("}\n");
        output
    }
    fn compile_type(&self, t: &grammar::KiroType) -> String {
        match t {
            grammar::KiroType::Num => "f64".to_string(), // Unified number type
            grammar::KiroType::Str => "String".to_string(),
            grammar::KiroType::Bool => "bool".to_string(),
            // The "Safe" Pointer: Atomic Reference Counted + Mutex
            // Since our 'Adr' type in grammar is generic-less, we assume generic runtime value
            // or we might need to cast. For now, let's map it to a dynamic pointer.
            grammar::KiroType::Adr => "std::sync::Arc<std::sync::Mutex<f64>>".to_string(), // Simplified for v1

            grammar::KiroType::Pipe => "KiroPipe<f64>".to_string(),

            // Recursive Generics
            grammar::KiroType::List(_, inner) => format!("Vec<{}>", self.compile_type(inner)),
            grammar::KiroType::Map(_, k, v) => format!(
                "std::collections::HashMap<{}, {}>",
                self.compile_type(k),
                self.compile_type(v)
            ),

            // Map Custom Types directly to Rust Struct names
            grammar::KiroType::Custom(s) => s.value.clone(),
        }
    }
    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            // 1. Compile Struct Definition
            Statement::StructDef { name, fields, .. } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name.value, self.compile_type(&f.field_type)))
                    .collect();

                // We add #[derive(Clone, Debug, PartialEq)] and impl KiroGet
                format!(
                    "#[derive(Clone, Debug, PartialEq)]\nstruct {0} {{ {1} }}\nimpl KiroGet for {0} {{ type Inner = Self; fn kiro_get<R>(&self, f: impl FnOnce(&Self::Inner) -> R) -> R {{ f(self) }} }}",
                    name.value,
                    field_strs.join(", ")
                )
            }
            Statement::Assignment {
                var_kw,
                ident,
                value,
                ..
            } => {
                let val_str = self.compile_expr(value);
                let is_new = !self.known_vars.contains(&ident);
                let user_wrote_var = var_kw.is_some();

                // LOGIC: Kiro -> Rust Mapping
                if is_new {
                    self.known_vars.insert(ident.clone());
                    if user_wrote_var {
                        // Kiro: var x = 10  -> Rust: let mut x = 10;
                        format!("let mut {} = {};", ident, val_str)
                    } else {
                        // Kiro: y = 20      -> Rust: let y = 20;
                        format!("let {} = {};", ident, val_str)
                    }
                } else {
                    // Kiro: x = 30      -> Rust: x = 30;
                    format!("{} = {};", ident, val_str)
                }
            }
            Statement::Print(_, expr) => {
                let val = self.compile_expr(expr);
                format!("println!(\"{{}}\", {});", val)
            }
            Statement::On {
                condition,
                body,
                else_clause,
                ..
            } => {
                let cond_str = self.compile_expr(condition);
                let body_str = self.compile_block(body);

                let else_str = match else_clause {
                    Some(clause) => format!("else {}", self.compile_block(clause.body)),
                    None => String::new(),
                };

                format!("if {} {} {}", cond_str, body_str, else_str)
            }
            Statement::LoopOn {
                condition, body, ..
            } => {
                let cond_str = self.compile_expr(condition);
                let body_str = self.compile_block(body);
                format!("while {} {}", cond_str, body_str)
            }
            // 2. Iterator Loop -> Rust 'for' with injected logic
            Statement::LoopIter {
                iterator,
                iterable,
                step,
                filter,
                body,
                else_clause,
                ..
            } => {
                let range_str = self.compile_expr(iterable);

                // Handle "per 5" -> .step_by(5)
                let iter_call = if let Some(s) = step {
                    let step_val = self.compile_expr(s.value);
                    format!("{}.kiro_iter().step_by({} as usize)", range_str, step_val)
                } else {
                    format!("{}.kiro_iter()", range_str)
                };

                // Handle "on (cond)" -> Inject 'if/else' inside the loop body
                let inner_logic = if let Some(f) = filter {
                    let cond_str = self.compile_expr(f.condition);
                    let true_block = self.compile_block(body);
                    let false_block = match else_clause {
                        Some(off) => format!("else {}", self.compile_block(off.body)),
                        None => String::new(),
                    };

                    // The body of the loop becomes an IF statement
                    format!("{{ if {} {} {} }}", cond_str, true_block, false_block)
                } else {
                    // No filter? Just run the block normally.
                    self.compile_block(body)
                };

                // Implicit Mutability Rule:
                self.known_vars.insert(iterator.clone());

                format!(
                    "for {}_temp in {} {{ let {} = {}_temp.as_kiro(); {} }}",
                    iterator, iter_call, iterator, iterator, inner_logic
                )
            }
            Statement::FunctionDef {
                name, params, body, ..
            } => {
                // In Kiro, functions are async by default (for 'run')
                // We ignore 'pure' in transpilation (it's a safety check, not a syntax change)

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.compile_type(&p.command_type)))
                    .collect();

                let body_str = self.compile_block(body);

                // We force return type to i64 for now (since we only have numbers)
                // In the future, this will be 'RuntimeVal' or inferred.
                format!("async fn {}({}) {}", name, param_strs.join(", "), body_str)
            }

            // 2. Expression Statement (Standard Call on its own line)
            Statement::ExprStmt(expr) => {
                let val = self.compile_expr(expr);
                format!("{};", val)
            }
            Statement::Give(_, channel, value) => {
                let ch = self.compile_expr(channel);
                let val = self.compile_expr(value);
                // We use unwrap() because if the receiver is closed, panic is appropriate for now
                format!("{}.tx.send({}).await.unwrap();", ch, val)
            }

            // 4. Close -> .tx.close()
            Statement::Close(_, channel) => {
                let ch = self.compile_expr(channel);
                format!("{}.tx.close();", ch)
            }
        }
    }

    fn compile_expr(&self, expr: Expression) -> String {
        match expr {
            Expression::Variable(v) => v.value,

            // 2. Compile Struct Init
            Expression::StructInit(name, _, fields, _) => {
                let init_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name.value, self.compile_expr(f.value.clone())))
                    .collect();

                format!("{} {{ {} }}", name.value, init_strs.join(", "))
            }

            // 3. Compile Field Access
            // Rust supports auto-deref for dot operator on many types.
            // If it's a raw struct: user.name works.
            // If it's a reference:            // 3. Compile Field Access
            Expression::FieldAccess(target, _, field) => {
                // Use helper trait for Auto-Deref
                format!(
                    "{}.kiro_get(|v| v.{}.clone())",
                    self.compile_expr(*target),
                    field.value
                )
            }

            // FIXED: Unwrap NumberVal
            Expression::Number(num_val) => {
                let n: f64 = num_val.value.parse().unwrap();
                if n.fract() == 0.0 {
                    format!("{:.1}", n)
                } else {
                    n.to_string()
                }
            }

            // FIXED: Unwrap StringVal
            Expression::StringLit(s) => format!("String::from({})", s.value),
            Expression::BoolLit(b) => match b {
                grammar::BoolVal::True(_) => "true".to_string(),
                grammar::BoolVal::False(_) => "false".to_string(),
            },
            Expression::PipeInit(_, _type) => {
                // Ignore type for now, assume f64 (or use it if we implemented generics)
                "{ let (tx, rx) = async_channel::unbounded(); KiroPipe { tx, rx } }".to_string()
            }

            // 6. Take -> .rx.recv().await
            Expression::Take(_, channel) => {
                let ch = self.compile_expr(*channel);
                format!("{}.rx.recv().await.unwrap()", ch)
            }
            // New Pointer Logic
            Expression::Ref(_, target) => {
                // ref x  ->  Arc::new(Mutex::new(x))
                let val = self.compile_expr(*target);
                format!("std::sync::Arc::new(std::sync::Mutex::new({}))", val)
            }
            Expression::Deref(_, target) => {
                // deref x  ->  *(x.lock().unwrap())
                let ptr = self.compile_expr(*target);
                // We lock the mutex, unwrap the result (crash on poison), and dereference the guard
                format!("*({}.lock().unwrap())", ptr)
            }

            // 2. List Init -> vec![...]
            Expression::ListInit(_, _, _, items, _) => {
                let elems: Vec<String> =
                    items.iter().map(|e| self.compile_expr(e.clone())).collect();
                format!("vec![{}]", elems.join(", "))
            }

            // 3. Map Init -> HashMap::from(...)
            Expression::MapInit(_, _, _, _, pairs, _) => {
                let entries: Vec<String> = pairs
                    .iter()
                    .map(|p| {
                        format!(
                            "({}, {})",
                            self.compile_expr(p.key.clone()),
                            self.compile_expr(p.value.clone())
                        )
                    })
                    .collect();
                format!("std::collections::HashMap::from([{}])", entries.join(", "))
            }

            // 4. 'at' Command
            Expression::At(col, _, key) => {
                format!(
                    "{}.kiro_at({})",
                    self.compile_expr(*col),
                    self.compile_expr(*key)
                )
            }

            // 5. 'push' Command
            Expression::Push(list, _, val) => {
                format!(
                    "{}.push({})",
                    self.compile_expr(*list),
                    self.compile_expr(*val)
                )
            }

            Expression::Add(lhs, _, rhs) => format!(
                "({}.kiro_add({}))",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Len(_, expr) => {
                format!("{}.kiro_len()", self.compile_expr(*expr))
            }
            Expression::Sub(lhs, _, rhs) => format!(
                "({} - {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Mul(lhs, _, rhs) => format!(
                "({} * {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Div(lhs, _, rhs) => format!(
                "({} / {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Eq(lhs, _, rhs) => format!(
                "({} == {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Neq(lhs, _, rhs) => format!(
                "({} != {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Gt(lhs, _, rhs) => format!(
                "({} > {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Lt(lhs, _, rhs) => format!(
                "({} < {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Geq(lhs, _, rhs) => format!(
                "({} >= {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Leq(lhs, _, rhs) => format!(
                "({} <= {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
            Expression::Range(start, _, end) => {
                format!(
                    "(({} as i64)..({} as i64))",
                    self.compile_expr(*start),
                    self.compile_expr(*end)
                )
            }
            // 3. Normal Call -> await
            Expression::Call(func, _, args, _) => {
                let func_name = self.compile_expr(*func);
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| format!("({}).clone()", self.compile_expr(a.clone())))
                    .collect();

                format!("{}({}).await", func_name, arg_strs.join(", "))
            }

            // 4. Run Call -> tokio::spawn
            Expression::RunCall(_, call_expr) => {
                // call_expr is the "foo(x)" part.
                // We need to strip the ".await" that compile_expr normally adds to calls!
                // This is a bit tricky. Let's handle it manually:

                if let Expression::Call(func, _, args, _) = *call_expr {
                    let func_name = self.compile_expr(*func);
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|a| format!("({}).clone()", self.compile_expr(a.clone())))
                        .collect();

                    // Spawn logic:
                    format!("tokio::spawn({}({}))", func_name, arg_strs.join(", "))
                } else {
                    "/* Error: run must be followed by a function call */".to_string()
                }
            }
        }
    }
    fn compile_block(&mut self, block: grammar::Block) -> String {
        let len = block.statements.len();
        let mut lines = Vec::new();

        for (i, stmt) in block.statements.iter().enumerate() {
            let mut line = self.compile_statement(stmt.clone());

            // LOGIC: If this is the LAST statement...
            if i == len - 1 {
                // Check if it's an ExprStmt (standalone expression)
                if let grammar::Statement::ExprStmt(_) = stmt {
                    // Remove the trailing semicolon if it exists
                    if line.ends_with(';') {
                        line.pop();
                    }
                }
            }
            lines.push(line);
        }
        format!("{{\n{}\n}}", lines.join("\n"))
    }
}
