use crate::grammar::{self, Statement};
use std::collections::HashMap;

pub mod expression;
pub mod statement;
pub mod values;

use values::{RuntimeVal, Value};

#[derive(Debug, Clone)]
pub enum StmtResult {
    Normal(RuntimeVal),
    Return(RuntimeVal),
    Break,
    Continue,
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
            let res = self.execute_statement(statement)?;
            // If top-level statement returns Return/Break/Continue, that's an error
            // (or we could just exit logic, but standard is scripts only return via exit)
            match res {
                StmtResult::Normal(_) => {}
                StmtResult::Return(_) => return Ok(()), // Allow script to return
                StmtResult::Break | StmtResult::Continue => {
                    return Err("Cannot break/continue outside of loop".to_string());
                }
            }
        }
        Ok(())
    }
}
