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
        }
    }
    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
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
                let mut range_str = self.compile_expr(iterable);

                // Handle "per 5" -> .step_by(5)
                if let Some(s) = step {
                    let step_val = self.compile_expr(s.value);
                    // Wrap in parens to ensure (0..10).step_by(5)
                    range_str = format!("({}).step_by({} as usize)", range_str, step_val);
                }

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
                // Since this is a loop, we register 'iterator' as a known var to avoid conflicts,
                // but in Rust loops, the iterator is declared implicitly.
                self.known_vars.insert(iterator.clone());

                format!("for {} in {} {}", iterator, range_str, inner_logic)
            }
            Statement::FunctionDef {
                pure_kw,
                name,
                params,
                body,
                ..
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
                format!(
                    "async fn {}({}) -> i64 {}",
                    name,
                    param_strs.join(", "),
                    body_str
                )
            }

            // 2. Expression Statement (Standard Call on its own line)
            Statement::ExprStmt(expr) => {
                let val = self.compile_expr(expr);
                format!("{};", val)
            }
        }
    }

    fn compile_expr(&self, expr: Expression) -> String {
        match expr {
            Expression::Variable(v) => v.value,

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
            Expression::Add(lhs, _, rhs) => format!(
                "({} + {})",
                self.compile_expr(*lhs),
                self.compile_expr(*rhs)
            ),
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
                    "({}..{})",
                    self.compile_expr(*start),
                    self.compile_expr(*end)
                )
            }
            // 3. Normal Call -> await
            Expression::Call(func, _, args, _) => {
                let func_name = self.compile_expr(*func);
                let arg_strs: Vec<String> =
                    args.iter().map(|a| self.compile_expr(a.clone())).collect();

                format!("{}({}).await", func_name, arg_strs.join(", "))
            }

            // 4. Run Call -> tokio::spawn
            Expression::RunCall(_, call_expr) => {
                // call_expr is the "foo(x)" part.
                // We need to strip the ".await" that compile_expr normally adds to calls!
                // This is a bit tricky. Let's handle it manually:

                if let Expression::Call(func, _, args, _) = *call_expr {
                    let func_name = self.compile_expr(*func);
                    let arg_strs: Vec<String> =
                        args.iter().map(|a| self.compile_expr(a.clone())).collect();

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
