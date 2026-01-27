use crate::grammar::grammar;
use std::collections::HashSet;

pub mod expression;
pub mod statement;
pub mod types;

pub struct Compiler {
    known_vars: HashSet<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            known_vars: HashSet::new(),
        }
    }
    pub fn compile(&mut self, program: grammar::Program) -> String {
        let mut output = String::new();
        output.push_str("#![allow(unused_mut, unused_variables, unused_parens)]\n");
        // 1. Import async_channel
        output.push_str("use async_channel;\n");

        // 2. Define the Pipe Wrapper
        // We use a struct so we can pass a single 'p' variable that holds both ends.
        output.push_str(
            r#"
            #[derive(Clone)]
            struct KiroPipe<T> {
                tx: async_channel::Sender<T>,
                rx: async_channel::Receiver<T>,
            }

            // --- HELPER TRAIT FOR AUTO-DEREF ---
            trait KiroGet {
                type Inner;
                fn kiro_get<R>(&self, f: impl FnOnce(&Self::Inner) -> R) -> R;
            }

            // Pointer (e.g., Arc<Mutex<User>>)
            impl<T> KiroGet for std::sync::Arc<std::sync::Mutex<T>> {
                type Inner = T;
                fn kiro_get<R>(&self, f: impl FnOnce(&T) -> R) -> R {
                    let guard = self.lock().unwrap();
                    f(&*guard)
                }
            }

            // --- KIRO AT TRAIT (Access Command) ---
            trait KiroAt<I, O> { fn kiro_at(&self, index: I) -> O; }

            // List Implementation: list at 0
            impl<T: Clone> KiroAt<f64, T> for Vec<T> {
                fn kiro_at(&self, index: f64) -> T {
                    self.get(index as usize).cloned().expect("Index out of bounds")
                }
            }

            // Map Implementation: map at "key"
            impl<K, V> KiroAt<K, V> for std::collections::HashMap<K, V> 
            where K: std::hash::Hash + Eq + Clone, V: Clone {
                fn kiro_at(&self, key: K) -> V {
                    self.get(&key).cloned().expect("Key not found")
                }
            }

            // --- KIRO ADD (String Concatenation vs Math) ---
            trait KiroAdd<Rhs = Self> { type Output; fn kiro_add(self, rhs: Rhs) -> Self::Output; }
            impl KiroAdd for f64 { type Output = f64; fn kiro_add(self, rhs: f64) -> f64 { self + rhs } }
            impl KiroAdd for String { type Output = String; fn kiro_add(self, rhs: String) -> String { format!("{}{}", self, rhs) } }

            // --- KIRO LEN ---
            trait KiroLen { fn kiro_len(&self) -> f64; }
            impl<T> KiroLen for Vec<T> { fn kiro_len(&self) -> f64 { self.len() as f64 } }
            impl<K, V> KiroLen for std::collections::HashMap<K, V> { fn kiro_len(&self) -> f64 { self.len() as f64 } }
            impl KiroLen for String { fn kiro_len(&self) -> f64 { self.len() as f64 } }

            // --- KIRO ITER (Looping) ---
            trait KiroIter { type Item; type IntoIter: Iterator<Item = Self::Item>; fn kiro_iter(self) -> Self::IntoIter; }
            impl KiroIter for std::ops::Range<i64> { type Item = i64; type IntoIter = std::ops::Range<i64>; fn kiro_iter(self) -> Self::IntoIter { self } }
            impl<T> KiroIter for Vec<T> { type Item = T; type IntoIter = std::vec::IntoIter<T>; fn kiro_iter(self) -> Self::IntoIter { self.into_iter() } }
            impl KiroIter for String { type Item = char; type IntoIter = std::vec::IntoIter<char>; fn kiro_iter(self) -> Self::IntoIter { self.chars().collect::<Vec<_>>().into_iter() } }

            // --- AS KIRO LOOP VAR (Type Correction) ---
            trait AsKiroLoopVar { type Out; fn as_kiro(self) -> Self::Out; }
            impl AsKiroLoopVar for i64 { type Out = f64; fn as_kiro(self) -> f64 { self as f64 } }
            impl AsKiroLoopVar for f64 { type Out = f64; fn as_kiro(self) -> f64 { self } }
            impl AsKiroLoopVar for char { type Out = String; fn as_kiro(self) -> String { self.to_string() } }
            impl AsKiroLoopVar for String { type Out = String; fn as_kiro(self) -> String { self } }

        "#,
        );
        output.push_str("#[tokio::main]\nasync fn main(){\n");
        for statement in program.statements {
            let line = self.compile_statement(statement);
            output.push_str(&format!("\t{}\n", line));
        }
        output.push_str("}\n");
        output
    }
}
