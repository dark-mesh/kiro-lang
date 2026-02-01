use super::Interpreter;
use super::StatementResult; // New Enum
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
    pub fn execute_statement(&mut self, statement: Statement) -> Result<StatementResult, String> {
        match statement {
            // Error definitions register the type and description
            Statement::ErrorDef {
                name, description, ..
            } => {
                let desc = description.map(|d| d.value.value).unwrap_or_default();
                self.error_types.insert(name.clone(), desc);
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            // Struct definitions are just Declarations, no runtime effect in interpreter
            Statement::StructDef { .. } => Ok(StatementResult::Normal(RuntimeVal::Void)),
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
                Ok(StatementResult::Normal(RuntimeVal::Void))
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
                            Ok(StatementResult::Normal(RuntimeVal::Void))
                        } else {
                            // NEW: Immutable Declaration (First Assignment)
                            // If it doesn't exist, we create it as IMMUTABLE.
                            // "const x = 10" is achieved by "x = 10"
                            self.env.insert(
                                name,
                                Value {
                                    data: new_val,
                                    is_mutable: false, // Immutable by default!
                                },
                            );
                            Ok(StatementResult::Normal(RuntimeVal::Void))
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

                        Ok(StatementResult::Normal(RuntimeVal::Void))
                    }
                    _ => Err("Invalid left-hand side for assignment.".to_string()),
                }
            }

            // 3. Control Flow
            Statement::Return(_, expr_opt) => {
                if let Some(expr) = expr_opt {
                    let val = self.eval_expr(expr)?;
                    Ok(StatementResult::Return(val))
                } else {
                    Ok(StatementResult::Return(RuntimeVal::Void))
                }
            }
            Statement::Break(_) => Ok(StatementResult::Break),
            Statement::Continue(_) => Ok(StatementResult::Continue),

            Statement::On {
                condition,
                body,
                else_clause,
                error_clauses,
                ..
            } => {
                let val = self.eval_expr(condition)?;

                // Helper to flatten ErrorClauseList into Vec<&grammar::ErrorClause>
                fn flatten_clauses(list: &grammar::ErrorClauseList) -> Vec<&grammar::ErrorClause> {
                    let mut result = vec![&list.first];
                    if let Some(ref rest) = list.rest {
                        result.extend(flatten_clauses(rest));
                    }
                    result
                }

                // Check if value is an Error
                if let RuntimeVal::Error(ref err_name, ref err_desc) = val {
                    // Try to match against error clauses in order
                    if let Some(ref error_list) = error_clauses {
                        let clauses = flatten_clauses(error_list);
                        for clause in clauses.iter() {
                            // If error_type is None, it's a catch-all
                            let matches = clause.error_type.is_none()
                                || clause.error_type.as_ref() == Some(err_name);
                            if matches {
                                let result = self.execute_block(clause.body.clone())?;
                                // If block returned normally with Void, implicitly return the error
                                match result {
                                    StatementResult::Normal(RuntimeVal::Void) => {
                                        return Ok(StatementResult::Return(RuntimeVal::Error(
                                            err_name.clone(),
                                            err_desc.clone(),
                                        )));
                                    }
                                    other => return Ok(other),
                                }
                            }
                        }
                    }
                    // If no clause matched, return the error as-is (propagation)
                    return Ok(StatementResult::Return(val));
                }

                // Standard truthy check for non-error values
                if val.is_truthy() {
                    self.execute_block(body)
                } else {
                    if let Some(clause) = else_clause {
                        self.execute_block(clause.body)
                    } else {
                        Ok(StatementResult::Normal(RuntimeVal::Void))
                    }
                }
            }
            Statement::LoopOn {
                condition, body, ..
            } => {
                // While condition evaluates to True (1)
                loop {
                    // Re-evaluate condition each iteration
                    let val = self.eval_expr(condition.clone())?;

                    if !val.is_truthy() {
                        break;
                    }
                    let res = self.execute_block(body.clone())?;
                    match res {
                        StatementResult::Normal(_) => {}
                        StatementResult::Continue => {} // Loop again
                        StatementResult::Break => break,
                        StatementResult::Return(v) => return Ok(StatementResult::Return(v)),
                    }
                }
                Ok(StatementResult::Normal(RuntimeVal::Void))
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
                            StatementResult::Normal(_) => {}
                            StatementResult::Continue => {}
                            StatementResult::Break => break_loop = true,
                            StatementResult::Return(v) => {
                                // Must restore env BEFORE returning!
                                self.env = parent_env;
                                return Ok(StatementResult::Return(v));
                            }
                        }
                    } else if let Some(off) = &else_clause {
                        let res = self.execute_block(off.body.clone())?;
                        match res {
                            StatementResult::Normal(_) => {}
                            StatementResult::Continue => {}
                            StatementResult::Break => break_loop = true,
                            StatementResult::Return(v) => {
                                self.env = parent_env;
                                return Ok(StatementResult::Return(v));
                            }
                        }
                    }

                    self.env = parent_env;

                    if break_loop {
                        break;
                    }
                }
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            Statement::Print(_, expr) => {
                if self.in_pure_mode {
                    return Err("Pure Function Error: 'print' is forbidden.".to_string());
                }
                let val = self.eval_expr(expr)?;
                println!("{}", val);
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            Statement::ExprStmt(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(StatementResult::Normal(val))
            }
            stmt @ Statement::FunctionDef { .. } => {
                if let Statement::FunctionDef { name, .. } = &stmt {
                    let func_name = name.clone();
                    self.functions.insert(func_name.clone(), stmt);
                    println!("✨ Registered Function: {}", func_name);
                }
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            // Rust-backed function declaration (register for lookup)
            stmt @ Statement::RustFnDecl { .. } => {
                if let Statement::RustFnDecl { name, .. } = &stmt {
                    let func_name = name.clone();
                    self.functions.insert(func_name.clone(), stmt);
                    println!(
                        "✨ Registered Rust Function: {} (compile to run)",
                        func_name
                    );
                }
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            // 1. Give (Sync Send)
            Statement::Give(_, channel_expr, value_expr) => {
                if self.in_pure_mode {
                    return Err("Pure Function Error: 'give' is forbidden.".to_string());
                }
                let chan = self.eval_expr(channel_expr)?;
                let val = self.eval_expr(value_expr)?.as_float()?;

                if let RuntimeVal::Pipe(tx, _) = chan {
                    tx.send(val)
                        .map_err(|_| "Pipe Error: Receiver closed".to_string())?;
                } else {
                    return Err("Runtime Error: 'give' expects a pipe.".to_string());
                }
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }

            // 2. Close (Drop Sender)
            Statement::Close(_, _channel_expr) => {
                println!("⚠️ [Interpreter] 'close' is a no-op in test mode.");
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
            // 7. Import Logic
            Statement::Import { module_name, .. } => {
                // Resolve module path:
                // 1. If starts with "std_", look in src/kiro_std/{module_name}/std_{module_name}.kiro
                // 2. Otherwise, look in current directory as {name}.kiro
                let (source, filename) = if module_name.starts_with("std_") {
                    let module_suffix = &module_name[4..]; // Remove "std_" prefix
                    let asset_path = format!("{}/{}.kiro", module_suffix, module_name);
                    let content = crate::StdAssets::get(&asset_path)
                        .map(|f| std::str::from_utf8(f.data.as_ref()).unwrap().to_string())
                        .ok_or_else(|| {
                            format!(
                                "Standard library module '{}' not found in embedded assets",
                                module_name
                            )
                        })?;
                    (content, asset_path)
                } else {
                    let filename = format!("{}.kiro", module_name);
                    let content = std::fs::read_to_string(&filename)
                        .map_err(|_| format!("Module '{}' not found", filename))?;
                    (content, filename)
                };

                println!("📦 Importing {}...", filename);

                // B. Parse it
                // We need to access the parse function.
                // Since main.rs uses grammar::parse, and grammar is crate::grammar::grammar
                // We'll try crate::grammar::grammar::parse
                let program = crate::grammar::grammar::parse(&source)
                    .map_err(|e| format!("Parse error in {}: {:?}", filename, e))?;

                // C. Create a fresh Interpreter for the module
                // We need to use valid imports for Interpreter and Value
                let mut module_interp = Interpreter::new();

                // We need to call run, but run takes ownership of self usually?
                // Interpreter::run(&mut self, program)
                module_interp.run(program)?;

                // D. Extract everything (since 'Everything is Public')
                let mut exports = std::collections::HashMap::new();
                for (key, val) in module_interp.env {
                    exports.insert(key, val.data);
                }

                // E. Save as a Module in the current scope
                self.env.insert(
                    module_name,
                    Value {
                        data: RuntimeVal::Module(exports, module_interp.functions),
                        is_mutable: false,
                    },
                );
                Ok(StatementResult::Normal(RuntimeVal::Void))
            }
        }
    }
    pub fn execute_block(&mut self, block: grammar::Block) -> Result<StatementResult, String> {
        let mut last_val = RuntimeVal::Void;

        for stmt in block.statements {
            let res = self.execute_statement(stmt)?;
            match res {
                StatementResult::Normal(v) => last_val = v,
                // Bubble up control flow signals immediately!
                _ => return Ok(res),
            }
        }
        Ok(StatementResult::Normal(last_val))
    }
}
