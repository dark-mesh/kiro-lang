use std::collections::HashMap;
use crate::grammar::{self, Expression,Statement};

#[derive(Clone,Debug)]
struct Value {
    data: i64,
    is_mutable: bool
}
pub struct Interpreter {
    env: HashMap<String, Value>,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new()
        }
    }
    pub fn run(&mut self, program: grammar::Program){
        for statement in program.statements {
            match self.execute_statement(statement){
                Ok(_) => (),
                Err(e) => eprintln!("Runtime Error: {}",e)
            }
        }
    }
    fn execute_statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::Assignment { var_kw, ident, _eq, value, ..} => {
                let val = self.eval_expr(value)?;
                let user_wrote_var = var_kw.is_some();
                let existing_info = self.env.get(&ident).map(|v| v.is_mutable);
                match existing_info {
                    // CASE A: The Variable Already Exists
                    Some(is_mutable) => {
                        if user_wrote_var {
                            return Err(format!("ERROR: '{}' already exists. Don't use 'var' again.", ident));
                        }
                        if !is_mutable {
                            return Err(format!("ERROR: '{}' is immutable (read-only). You cannot change it.", ident));
                        }
                        // It's mutable, so we update it!
                        self.env.insert(ident.clone(), Value { data: val, is_mutable: true });
                        println!("🔄 Reassigned Mutable: {} = {}", ident, val);
                    }
                    
                    // CASE B: The Variable is New (Your Fix is Here)
                    None => {
                        // If 'var' is present -> Mutable. 
                        // If 'var' is MISSING -> Immutable (The Sane Default).
                        self.env.insert(ident.clone(), Value { 
                            data: val, 
                            is_mutable: user_wrote_var 
                        });
                        
                        let type_log = if user_wrote_var { "Mutable" } else { "Immutable" };
                        println!("✨ Defined New {}: {} = {}", type_log, ident, val);
                    }
                }
                Ok(())
            }
            Statement::On { condition, body, else_clause, .. } => {
                // 1. Evaluate the condition (returns 1 for True, 0 for False)
                let result = self.eval_expr(condition)?;

                if result != 0 {
                    // True: Run the main block
                    self.execute_block(body)?;
                } else {
                    // False: Run the 'off' block (if it exists)
                    if let Some(clause) = else_clause {
                        self.execute_block(clause.body)?;
                    }
                }
                Ok(())
            }
            _ => todo!()
        }
    }
    fn execute_block(&mut self, block: grammar::Block) -> Result<(), String> {
        for statement in block.statements {
            // Recursion: A block is just a list of statements!
            self.execute_statement(statement)?; 
        }
        Ok(())
    }
    fn eval_expr(&self, expr: Expression) -> Result<i64,String> {
        match expr {
            Expression::Number(n) => Ok(n),
            Expression::Variable(name) => {
                self.env.get(&name)
                    .map(|v| v.data)
                    .ok_or_else(|| format!("ERROR: Variable {} not found.",name))
            }
            Expression::Add(lhs, _, rhs) => Ok(self.eval_expr(*lhs)? + self.eval_expr(*rhs)?),
            Expression::Sub(lhs, _, rhs) => Ok(self.eval_expr(*lhs)? - self.eval_expr(*rhs)?),
            Expression::Mul(lhs, _, rhs) => Ok(self.eval_expr(*lhs)? * self.eval_expr(*rhs)?),
            Expression::Div(lhs, _, rhs) => Ok(self.eval_expr(*lhs)? / self.eval_expr(*rhs)?),
            Expression::Gt(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? > self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            Expression::Lt(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? < self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            Expression::Eq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? == self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            Expression::Neq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? != self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            Expression::Geq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? >= self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            Expression::Leq(lhs, _, rhs) => {
                let val = if self.eval_expr(*lhs)? <= self.eval_expr(*rhs)? { 1 } else { 0 };
                Ok(val)
            },
            _ => todo!()
        }
    }
}