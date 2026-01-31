use super::Compiler;

use crate::grammar::grammar::{self, Expression};

impl Compiler {
    pub fn compile_expr(&self, expr: Expression) -> String {
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
                // Check if the target is a known module (e.g., "math")
                if let Expression::Variable(v) = &*target {
                    if self.imported_modules.contains(&v.value) {
                        // It is a module! Use Rust's '::' syntax
                        return format!("{}::{}", v.value, field.value);
                    }
                }

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

            // Adr Init (Lazy / Void)
            Expression::AdrInit(_, inner) => {
                if let grammar::KiroType::Void = inner {
                    // adr void -> usize 0
                    "0usize".to_string()
                } else {
                    // adr T -> Option<...> = None
                    let type_str = crate::compiler::types::compile_type(&inner);
                    format!(
                        "Option::<std::sync::Arc<std::sync::Mutex<{}>>>::None",
                        type_str
                    )
                }
            }

            // Pipe Init
            Expression::PipeInit(_, pipe_type) => {
                let inner_type = crate::compiler::types::compile_type(&pipe_type);
                if let grammar::KiroType::Void = pipe_type {
                    // KiroPipe<()>
                    "{ let (tx, rx) = async_channel::unbounded(); KiroPipe::<()> { tx, rx } }"
                        .to_string()
                } else {
                    format!(
                        "{{ let (tx, rx) = async_channel::unbounded(); KiroPipe::<{}> {{ tx, rx }} }}",
                        inner_type
                    )
                }
            }

            // 6. Take -> .rx.recv().await
            Expression::Take(_, channel) => {
                let ch = self.compile_expr(*channel);
                format!("{}.rx.recv().await.unwrap()", ch)
            }
            // New Pointer Logic
            Expression::Ref(_, target) => {
                // ref x  ->  Some(Arc::new(Mutex::new(x)))
                let val = self.compile_expr(*target);
                format!("Some(std::sync::Arc::new(std::sync::Mutex::new({})))", val)
            }
            Expression::Deref(_, target) => {
                // deref x
                // If x is regular Adr (Option<Arc...>), we unwrap.
                // If x is Adr<Void> (usize), we panic or forbid?
                // The implementation plan says "Prohibitions (Strict Checks): Dereference: Check if target is Adr(Void)."
                // NOTE: We don't have type info of expressions easily available here in `compile_expr` without a type checker step.
                // However, Rust compilation will fail if we try to unwrap a usize.
                // Standard Adr: x.as_ref().unwrap().lock().unwrap()
                let ptr = self.compile_expr(*target);
                format!(
                    "*({}.as_ref().expect(\"Dereferencing Void/Null Pointer\").lock().unwrap())",
                    ptr
                )
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

    pub fn compile_lvalue(&self, expr: Expression) -> String {
        match expr {
            Expression::Variable(v) => v.value,
            Expression::FieldAccess(target, _, field) => {
                format!("{}.{}", self.compile_lvalue(*target), field.value)
            }
            Expression::Deref(_, target) => {
                format!("*({}.lock().unwrap())", self.compile_expr(*target))
            }
            _ => panic!("Invalid lvalue: {:?}", expr),
        }
    }
}
