use super::Compiler;
use super::types::compile_type;
use crate::grammar::grammar::{self, Statement};

impl Compiler {
    pub fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            // 1. Compile Struct Definition
            Statement::StructDef { name, fields, .. } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name.value, compile_type(&f.field_type)))
                    .collect();

                // We add #[derive(Clone, Debug, PartialEq)] and impl KiroGet
                format!(
                    "#[derive(Clone, Debug, PartialEq)]\nstruct {0} {{ {1} }}\nimpl KiroGet for {0} {{ type Inner = Self; fn kiro_get<R>(&self, f: impl FnOnce(&Self::Inner) -> R) -> R {{ f(self) }} }}",
                    name.value,
                    field_strs.join(", ")
                )
            }
            // 1. Variable Declaration
            Statement::VarDecl { ident, value, .. } => {
                let val_str = self.compile_expr(value);
                self.known_vars.insert(ident.clone());
                // In Kiro, vars are mutable by default
                format!("let mut {} = {};", ident, val_str)
            }

            // 2. Assignment (Mutation)
            Statement::AssignStmt { lhs, rhs, .. } => {
                let rhs_str = self.compile_expr(rhs);
                let lhs_str = self.compile_lvalue(lhs);
                format!("{} = {};", lhs_str, rhs_str)
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
                self.known_vars.insert(iterator.clone());

                format!(
                    "for {}_temp in {} {{ let {} = {}_temp.as_kiro(); {} }}",
                    iterator, iter_call, iterator, iterator, inner_logic
                )
            }
            Statement::FunctionDef {
                name, params, body, ..
            } => {
                // In Kiro, functions are async by default (for 'run')
                // We ignore 'pure' in transpilation (it's a safety check, not a syntax change)

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, compile_type(&p.command_type)))
                    .collect();

                let body_str = self.compile_block(body);

                // We force return type to f64 for now (since we only have numbers)
                // We append "; 0.0" to ensure the block returns a float even if it ends with print/void
                format!(
                    "async fn {}({}) -> f64 {{ {}; 0.0 }}",
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
            Statement::Give(_, channel, value) => {
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
                let val = self.compile_expr(expr);
                format!("return {};", val)
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
