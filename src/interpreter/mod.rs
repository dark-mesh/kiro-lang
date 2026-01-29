use crate::grammar::{self, Statement};
use std::collections::HashMap;

pub mod expression;
pub mod statement;
pub mod values;

use values::{RuntimeVal, Value};

#[derive(Debug, Clone)]
pub enum StatementResult {
    Normal(RuntimeVal),
    Return(RuntimeVal),
    Break,
    Continue,
}

pub struct Interpreter {
    pub env: HashMap<String, Value>,
    pub functions: HashMap<String, Statement>,
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
            let res = self.execute_statement(statement)?;
            // If top-level statement returns Return/Break/Continue, that's an error
            // (or we could just exit logic, but standard is scripts only return via exit)
            match res {
                StatementResult::Normal(_) => {}
                StatementResult::Return(_) => return Ok(()), // Allow script to return
                StatementResult::Break | StatementResult::Continue => {
                    return Err("Cannot break/continue outside of loop".to_string());
                }
            }
        }
        Ok(())
    }
}
