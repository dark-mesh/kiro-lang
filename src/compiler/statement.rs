use super::Compiler;
use super::types::compile_type;
use crate::grammar::grammar::{self, Statement};

impl Compiler {
    pub fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            // Error Definition: error NotFound = "Description"
            Statement::ErrorDef {
                name, description, ..
            } => {
                // d.value.value contains the raw string WITH quotes, we need to strip them
                let desc = description
                    .map(|d| d.value.value.trim_matches('"').to_string())
                    .unwrap_or_else(|| name.clone());
                // Generate a helper function that creates an anyhow error
                format!(
                    "fn kiro_error_{name}() -> anyhow::Error {{ anyhow::anyhow!(\"{desc}\").context(\"{name}\") }}"
                )
            }
            // 1. Compile Struct Definition
            // 1. Compile Struct Definition
            Statement::StructDef { name, fields, .. } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("pub {}: {}", f.name.value, compile_type(&f.field_type)))
                    .collect();

                // We add #[derive(Clone, Debug, PartialEq)] and impl KiroGet
                format!(
                    "#[derive(Clone, Debug)]\npub struct {0} {{ {1} }}\nimpl KiroGet for {0} {{ type Inner = Self; fn kiro_get<R>(&self, f: impl FnOnce(&Self::Inner) -> R) -> R {{ f(self) }} }}",
                    name.value,
                    field_strs.join(", ")
                )
            }
            // 6. Import Statement
            Statement::Import { module_name, .. } => {
                self.imported_modules.insert(module_name.clone());
                format!("pub mod {};", module_name)
            }
            // 1. Variable Declaration
            Statement::VarDecl { ident, value, .. } => {
                let val_str = self.compile_expr(value);
                self.known_vars
                    .insert(ident.clone(), super::VarInfo { is_mutable: true });
                // In Kiro, vars are mutable by default
                format!("let mut {} = {};", ident, val_str)
            }

            // ... (Middle assignments kept same, just copying context) ...
            Statement::AssignStmt { lhs, rhs, .. } => {
                let rhs_str = self.compile_expr(rhs);

                match lhs {
                    grammar::Expression::Variable(v) => {
                        let name = v.value;
                        if let Some(info) = self.known_vars.get(&name) {
                            if info.is_mutable {
                                // Mutable Assignment
                                format!("{}.kiro_assign({});", name, rhs_str)
                            } else {
                                // Immutable Assignment -> Error
                                panic!(
                                    "Compiler Error: Cannot mutate immutable variable '{}'.",
                                    name
                                );
                            }
                        } else {
                            // Implicit Immutable Declaration (x = 10)
                            self.known_vars
                                .insert(name.clone(), super::VarInfo { is_mutable: false });
                            format!("let {} = {};", name, rhs_str)
                        }
                    }
                    _ => {
                        // Complex LValue (e.g. x.y = 10)
                        let lhs_str = self.compile_lvalue(lhs);
                        format!("{}.kiro_assign({});", lhs_str, rhs_str)
                    }
                }
            }
            Statement::Print(_, expr) => {
                if self.in_pure_context {
                    panic!("Pure Function Error: 'print' is forbidden.");
                }
                let val = self.compile_expr(expr);
                format!("println!(\"{{}}\", {});", val)
            }
            Statement::On {
                condition,
                body,
                else_clause,
                error_clause,
                ..
            } => {
                let cond_str = self.compile_expr(condition.clone());
                let body_str = self.compile_block(body);

                // If there is an error clause, generate a match on Result
                if let Some(clause) = error_clause {
                    // Compile block
                    let block_body = self.compile_block(clause.body.clone());

                    // Implicit Error Return Logic:
                    // If we are in a failable function, we propagate the error implicitly if not returned.
                    // If we are in a non-failable function (like main), we cannot propagate (return Err),
                    // so we assume the error is handled and fall through (or user explicitly returns/panics).
                    let clause_body = if self.in_failable_fn {
                        format!("{} return Err(__kiro_err);", block_body)
                    } else {
                        block_body // Fall through (Swallow error as handled)
                    };

                    let shadowing = if let grammar::Expression::Variable(v) = &condition {
                        format!("let {} = __kiro_val;", v.value)
                    } else {
                        String::new()
                    };

                    let err_branch = if let Some(err_type) = clause.error_type {
                        // For unhandled types (else branch of the error check)
                        let propagation = if self.in_failable_fn {
                            "return Err(__kiro_err);"
                        } else {
                            "panic!(\"Unhandled error: {}\", __kiro_err);"
                        };

                        format!(
                            "if __kiro_err.to_string().contains(\"{}\") {{ {} }} else {{ {} }}",
                            err_type, clause_body, propagation
                        )
                    } else {
                        // Catch-all
                        clause_body
                    };

                    format!(
                        "match {} {{ Ok(__kiro_val) => {{ {} {} }} Err(__kiro_err) => {{ {} }} }}",
                        cond_str, shadowing, body_str, err_branch
                    )
                } else {
                    // Standard if/else
                    let else_str = match else_clause {
                        Some(clause) => format!("else {}", self.compile_block(clause.body)),
                        None => String::new(),
                    };
                    format!("if ({}).kiro_truthy() {} {}", cond_str, body_str, else_str)
                }
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
                self.known_vars
                    .insert(iterator.clone(), super::VarInfo { is_mutable: false });

                format!(
                    "for {}_temp in {} {{ let {} = {}_temp.as_kiro(); {} }}",
                    iterator, iter_call, iterator, iterator, inner_logic
                )
            }
            Statement::FunctionDef {
                name,
                params,
                return_type,
                body,
                pure_kw,
                can_error,
                ..
            } => {
                let is_pure = pure_kw.is_some();
                self.functions.insert(
                    name.clone(),
                    super::FunctionInfo {
                        is_pure,
                        can_error: can_error.is_some(),
                    },
                );

                let old_context = self.in_pure_context;
                if is_pure {
                    self.in_pure_context = true;
                }

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, compile_type(&p.command_type)))
                    .collect();

                let can_error = can_error.is_some();
                let old_in_failable = self.in_failable_fn;
                if can_error {
                    self.in_failable_fn = true;
                }

                let body_str = self.compile_block(body);

                self.in_pure_context = old_context;
                self.in_failable_fn = old_in_failable;

                let ret_def = if let Some(rt) = return_type {
                    if let crate::grammar::grammar::KiroType::Void = rt {
                        "()".to_string()
                    } else {
                        compile_type(&rt)
                    }
                } else {
                    "()".to_string()
                };

                let (ret_type, final_body) = if can_error {
                    (
                        format!("anyhow::Result<{}>", ret_def),
                        format!("{{ let __kiro_res = {}; Ok(__kiro_res) }}", body_str),
                    )
                } else {
                    (ret_def, body_str)
                };

                format!(
                    "pub async fn {}({}) -> {} {}",
                    name,
                    param_strs.join(", "),
                    ret_type,
                    final_body
                )
            }

            // Rust-backed function (external glue)
            Statement::RustFnDecl {
                name,
                params,
                return_type,
                can_error,
                ..
            } => {
                self.functions.insert(
                    name.clone(),
                    super::FunctionInfo {
                        is_pure: false,
                        can_error: can_error.is_some(),
                    },
                );

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, compile_type(&p.command_type)))
                    .collect();

                let can_error = can_error.is_some();

                let ret_def = if let crate::grammar::grammar::KiroType::Void = return_type {
                    "()".to_string()
                } else {
                    compile_type(&return_type)
                };

                // Generate call to header glue
                let arg_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let args_vec = if arg_names.is_empty() {
                    "vec![]".to_string()
                } else {
                    format!(
                        "vec![{}]",
                        arg_names
                            .iter()
                            .map(|a| format!("kiro_runtime::RuntimeVal::from({}.clone())", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                let final_body = if can_error {
                    format!(
                        "{{ match header::{}({}) {{ Ok(v) => Ok(v.try_into()?), Err(e) => Err(anyhow::anyhow!(e.name).context(e.name)) }} }}",
                        name, args_vec
                    )
                } else {
                    format!(
                        "{{ header::{}({}).unwrap().try_into().unwrap() }}",
                        name, args_vec
                    )
                };

                let ret_type = if can_error {
                    format!("anyhow::Result<{}>", ret_def)
                } else {
                    ret_def
                };

                format!(
                    "pub async fn {}({}) -> {} {}",
                    name,
                    param_strs.join(", "),
                    ret_type,
                    final_body
                )
            }

            // 2. Expression Statement (Standard Call on its own line)
            Statement::ExprStmt(expr) => {
                let val = self.compile_expr(expr);
                format!("{};", val)
            }
            Statement::Give(_, channel, value) => {
                if self.in_pure_context {
                    panic!("Pure Function Error: 'give' is forbidden.");
                }
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
            // 3. Return -> return ...
            Statement::Return(_, expr) => {
                if let Some(e) = expr {
                    let val = self.compile_expr(e);
                    if self.in_failable_fn && !val.starts_with("Err(") {
                        // In failable context, wrap non-error returns in Ok(...)
                        // Unless it's already an Err(...) creation
                        format!("return Ok({});", val)
                    } else {
                        format!("return {};", val)
                    }
                } else {
                    if self.in_failable_fn {
                        "return Ok(());".to_string()
                    } else {
                        "return;".to_string()
                    }
                }
            }
            // 4. Break -> break
            Statement::Break(_) => "break;".to_string(),
            // 5. Continue -> continue
            Statement::Continue(_) => "continue;".to_string(),
        }
    }

    pub fn compile_block(&mut self, block: grammar::Block) -> String {
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
