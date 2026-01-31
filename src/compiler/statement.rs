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
                let cond_str = self.compile_expr(condition);
                let body_str = self.compile_block(body);

                // If there is an error clause, generate a match on Result
                if let Some(clause) = error_clause {
                    let clause_body = self.compile_block(clause.body);
                    let err_branch = if let Some(err_type) = clause.error_type {
                        format!(
                            "if __kiro_err.to_string().contains(\"{}\") {} else {{ panic!(\"Unhandled error: {{}}\", __kiro_err); }}",
                            err_type, clause_body
                        )
                    } else {
                        // Catch-all
                        clause_body
                    };

                    format!(
                        "match {} {{ Ok(__kiro_val) => {} Err(__kiro_err) => {{ {} }} }}",
                        cond_str, body_str, err_branch
                    )
                } else {
                    // Standard if/else
                    let else_str = match else_clause {
                        Some(clause) => format!("else {}", self.compile_block(clause.body)),
                        None => String::new(),
                    };
                    format!("if ({}) != 0.0 {} {}", cond_str, body_str, else_str)
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
                ..
            } => {
                let is_pure = pure_kw.is_some();
                self.functions
                    .insert(name.clone(), super::FunctionInfo { is_pure });

                let old_context = self.in_pure_context;
                if is_pure {
                    self.in_pure_context = true;
                }

                // In Kiro, functions are async by default (for 'run')
                // We ignore 'pure' in transpilation (it's a safety check, not a syntax change)

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, compile_type(&p.command_type)))
                    .collect();

                let body_str = self.compile_block(body);

                self.in_pure_context = old_context;

                let ret_type = if let Some(rt) = return_type {
                    if let crate::grammar::grammar::KiroType::Void = rt {
                        "()".to_string()
                    } else {
                        compile_type(&rt)
                    }
                } else {
                    "()".to_string()
                };

                // We append "; Default::default()" to try to satisfy return types if block is void-ish
                // But generally, the block should return the value.
                // For proper transpiring, we rely on Rust's implicit return.
                // However, adding a fallback for void/unit might be needed.
                // For now, let's trust the block returns the right thing or user wrote 'return'.
                // EXCEPT: Kiro semantics might allow implicit return of last expr.
                // Rust does too. compile_block returns a block "{ stmts }"
                format!(
                    "pub async fn {}({}) -> {} {}",
                    name,
                    param_strs.join(", "),
                    ret_type,
                    body_str
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
                    format!("return {};", val)
                } else {
                    "return;".to_string()
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
