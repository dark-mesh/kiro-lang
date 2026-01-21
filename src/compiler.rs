use std::collections::HashSet;
use crate::grammar::{self,Statement,Expression};

pub struct Compiler {
    known_vars: HashSet<String>,
}
impl Compiler {
    pub fn new() -> Self {
        Self {
            known_vars: HashSet::new()
        }
    }
    pub fn compile(&mut self, program: grammar::Program) -> String {
        let mut output = String::new();
        output.push_str("#![allow(unused_mut, unused_variables, unused_parens)]\n");
        output.push_str("#[tokio::main]\nasync fn main(){\n");
        for statement in program.statements {
            let line = self.compile_statement(statement);
            output.push_str(&format!("\t{}\n",line));
            }
        output.push_str("}\n");
        output
    }
    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::Assignment { var_kw, ident, value, .. } => {
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
            },
            Statement::On { condition, body, else_clause, .. } => {
                let cond_str = self.compile_expr(condition);
                let body_str = self.compile_block(body);

                let else_str = match else_clause {
                    Some(clause) => format!("else {}", self.compile_block(clause.body)),
                    None => String::new(),
                };

                format!("if {} {} {}", cond_str, body_str, else_str)
            }
            Statement::LoopOn { condition, body, .. } => {
                let cond_str = self.compile_expr(condition);
                let body_str = self.compile_block(body);
                format!("while {} {}", cond_str, body_str)
            }
            // 2. Iterator Loop -> Rust 'for' with injected logic
            Statement::LoopIter { iterator, iterable, step, filter, body, else_clause, .. } => {
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
        }
    }

    fn compile_expr(&self, expr: Expression) -> String {
        match expr {
            Expression::Number(n) => n.to_string(),
            Expression::Variable(name) => name,
            Expression::Add(lhs, _, rhs) => format!("({} + {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Sub(lhs, _, rhs) => format!("({} - {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Mul(lhs, _, rhs) => format!("({} * {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Div(lhs, _, rhs) => format!("({} / {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Eq(lhs, _, rhs) => format!("({} == {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Neq(lhs, _, rhs) => format!("({} != {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Gt(lhs, _, rhs) => format!("({} > {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Lt(lhs, _, rhs) => format!("({} < {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Geq(lhs, _, rhs) => format!("({} >= {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Leq(lhs, _, rhs) => format!("({} <= {})", self.compile_expr(*lhs), self.compile_expr(*rhs)),
            Expression::Range(start, _, end) => {
                format!("({}..{})", self.compile_expr(*start), self.compile_expr(*end))
            },
        }
    }
    fn compile_block(&mut self, block: grammar::Block) -> String {
        let mut lines = Vec::new();
        for statement in block.statements {
            lines.push(self.compile_statement(statement));
        }
        // Rust blocks return the last expression, but for now let's just use semicolons
        format!("{{\n{}\n}}", lines.join("\n")) 
    }
}