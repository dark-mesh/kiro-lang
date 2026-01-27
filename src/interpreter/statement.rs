use super::Interpreter;
use super::StmtResult; // New Enum
use super::values::{RuntimeVal, Value};
use crate::grammar::grammar::{self, Statement};

// Helper for Deep Updates
// Path is reversed: [z, y] means x.y.z
fn update_nested_field(
    current: &mut RuntimeVal,
    mut path: Vec<String>,
    new_val: RuntimeVal,
) -> Result<(), String> {
    let field_name = path.pop().ok_or("Invalid path")?;

    if path.is_empty() {
        // We reached the target field!
        match current {
            RuntimeVal::Struct(_, fields) => {
                fields.insert(field_name, new_val);
                Ok(())
            }
            _ => Err("Target is not a struct".to_string()),
        }
    } else {
        // Drill down deeper
        match current {
            RuntimeVal::Struct(_, fields) => {
                let next_val = fields
                    .get_mut(&field_name)
                    .ok_or_else(|| format!("Field '{}' not found", field_name))?;
                update_nested_field(next_val, path, new_val)
            }
            _ => Err("Cannot access field on non-struct".to_string()),
        }
    }
}

impl Interpreter {
    pub fn execute_statement(&mut self, statement: Statement) -> Result<StmtResult, String> {
        match statement {
            // Struct definitions are just Declarations, no runtime effect in interpreter
            Statement::StructDef { .. } => Ok(StmtResult::Normal(RuntimeVal::Void)),
            // 1. Variable Declaration
            Statement::VarDecl { ident, value, .. } => {
                let val = self.eval_expr(value)?;
                self.env.insert(
                    ident.clone(),
                    Value {
                        data: val,
                        is_mutable: true, // New vars are always mutable in Kiro 1.0 logic
                    },
                );
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }

            // 2. Assignment (Top-level OR Field)
            Statement::AssignStmt { lhs, rhs, .. } => {
                let new_val = self.eval_expr(rhs)?;

                match lhs {
                    // Simple: x = 10
                    crate::grammar::grammar::Expression::Variable(v) => {
                        let name = v.value;
                        if let Some(entry) = self.env.get_mut(&name) {
                            if !entry.is_mutable {
                                return Err(format!("ERROR: '{}' is immutable.", name));
                            }
                            entry.data = new_val;
                            Ok(StmtResult::Normal(RuntimeVal::Void))
                        } else {
                            Err(format!("ERROR: Variable '{}' not declared.", name))
                        }
                    }
                    // Complex: x.y.z = 10
                    crate::grammar::grammar::Expression::FieldAccess(target, _, field) => {
                        let mut path = vec![field.value];
                        let mut current = *target;

                        // Unwind the dot chain: x.y.z -> path=[z, y], root=x
                        while let crate::grammar::grammar::Expression::FieldAccess(
                            inner_target,
                            _,
                            inner_field,
                        ) = current
                        {
                            path.push(inner_field.value);
                            current = *inner_target;
                        }

                        // Now 'current' should be the variable (x)
                        let root_name = match current {
                            crate::grammar::grammar::Expression::Variable(v) => v.value,
                            _ => {
                                return Err(
                                    "Assignment target must start with a variable.".to_string()
                                );
                            }
                        };

                        // 2. Get Mutable Root
                        let entry = self
                            .env
                            .get_mut(&root_name)
                            .ok_or_else(|| format!("Variable '{}' not found", root_name))?;

                        if !entry.is_mutable {
                            return Err(format!("Variable '{}' is immutable.", root_name));
                        }

                        // 3. Drill down and Update
                        update_nested_field(&mut entry.data, path, new_val)?;

                        Ok(StmtResult::Normal(RuntimeVal::Void))
                    }
                    _ => Err("Invalid left-hand side for assignment.".to_string()),
                }
            }

            // 3. Control Flow
            Statement::Return(_, expr) => {
                let val = self.eval_expr(expr)?;
                Ok(StmtResult::Return(val))
            }
            Statement::Break(_) => Ok(StmtResult::Break),
            Statement::Continue(_) => Ok(StmtResult::Continue),

            Statement::On {
                condition,
                body,
                else_clause,
                ..
            } => {
                let result = self.eval_expr(condition)?.as_float()?;
                if result != 0.0 {
                    // Bubble up the result (could be Break/Return!)
                    self.execute_block(body)
                } else {
                    if let Some(clause) = else_clause {
                        self.execute_block(clause.body)
                    } else {
                        Ok(StmtResult::Normal(RuntimeVal::Void))
                    }
                }
            }
            Statement::LoopOn {
                condition, body, ..
            } => {
                // While condition evaluates to True (1)
                loop {
                    if self.eval_expr(condition.clone())?.as_float()? == 0.0 {
                        break;
                    }
                    let res = self.execute_block(body.clone())?;
                    match res {
                        StmtResult::Normal(_) => {}
                        StmtResult::Continue => {} // Loop again
                        StmtResult::Break => break,
                        StmtResult::Return(v) => return Ok(StmtResult::Return(v)),
                    }
                }
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }
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
                    RuntimeVal::String(s) => s
                        .chars()
                        .map(|c| RuntimeVal::String(c.to_string()))
                        .collect(),
                    _ => {
                        return Err(
                            "Loop Error: Can only loop over ranges, lists, or strings".to_string()
                        );
                    }
                };

                for item in items {
                    let parent_env = self.env.clone();

                    self.env.insert(
                        iterator.clone(),
                        Value {
                            data: item,
                            is_mutable: false,
                        },
                    );

                    let run_main = if let Some(f) = &filter {
                        self.eval_expr(f.condition.clone())?.as_float()? != 0.0
                    } else {
                        true
                    };

                    let mut break_loop = false;

                    if run_main {
                        let res = self.execute_block(body.clone())?;
                        match res {
                            StmtResult::Normal(_) => {}
                            StmtResult::Continue => {}
                            StmtResult::Break => break_loop = true,
                            StmtResult::Return(v) => {
                                // Must restore env BEFORE returning!
                                self.env = parent_env;
                                return Ok(StmtResult::Return(v));
                            }
                        }
                    } else if let Some(off) = &else_clause {
                        let res = self.execute_block(off.body.clone())?;
                        match res {
                            StmtResult::Normal(_) => {}
                            StmtResult::Continue => {}
                            StmtResult::Break => break_loop = true,
                            StmtResult::Return(v) => {
                                self.env = parent_env;
                                return Ok(StmtResult::Return(v));
                            }
                        }
                    }

                    self.env = parent_env;

                    if break_loop {
                        break;
                    }
                }
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }
            Statement::Print(_, expr) => {
                let val = self.eval_expr(expr)?;
                println!("{}", val);
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }
            Statement::ExprStmt(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(StmtResult::Normal(val))
            }
            stmt @ Statement::FunctionDef { .. } => {
                if let Statement::FunctionDef { name, .. } = &stmt {
                    let func_name = name.clone();
                    self.functions.insert(func_name.clone(), stmt);
                    println!("✨ Registered Function: {}", func_name);
                }
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }
            // 1. Give (Sync Send)
            Statement::Give(_, channel_expr, value_expr) => {
                let chan = self.eval_expr(channel_expr)?;
                let val = self.eval_expr(value_expr)?.as_float()?;

                if let RuntimeVal::Pipe(tx, _) = chan {
                    tx.send(val)
                        .map_err(|_| "Pipe Error: Receiver closed".to_string())?;
                } else {
                    return Err("Runtime Error: 'give' expects a pipe.".to_string());
                }
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }

            // 2. Close (Drop Sender)
            Statement::Close(_, _channel_expr) => {
                println!("⚠️ [Interpreter] 'close' is a no-op in test mode.");
                Ok(StmtResult::Normal(RuntimeVal::Void))
            }
        }
    }
    pub fn execute_block(&mut self, block: grammar::Block) -> Result<StmtResult, String> {
        let mut last_val = RuntimeVal::Void;

        for stmt in block.statements {
            let res = self.execute_statement(stmt)?;
            match res {
                StmtResult::Normal(v) => last_val = v,
                // Bubble up control flow signals immediately!
                _ => return Ok(res),
            }
        }
        Ok(StmtResult::Normal(last_val))
    }
}
