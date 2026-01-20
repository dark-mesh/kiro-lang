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
        }
    }
}