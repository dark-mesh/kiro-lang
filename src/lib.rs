pub mod interpreter;
pub mod compiler;
#[rust_sitter::grammar("kiro")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub statements: Vec<Statement>
    }
    pub enum Statement {
        Assignment {
            #[rust_sitter::leaf(text = "var")]
            var_kw: Option<()>,
            #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
            ident: String,
            #[rust_sitter::leaf(text = "=")]
            _eq: (),

            value: Expression,
        },
    }
    pub enum Expression {
        Variable(
            #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())] 
            String
        ),
        Number(
            #[rust_sitter::leaf(pattern=r"\d+", transform= |s| s.parse().unwrap())] i64
        ),
        #[rust_sitter::prec_left(2)]
        Mul(
            Box<Expression>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(2)]
        Div(
            Box<Expression>,
            #[rust_sitter::leaf(text = "/")] (),
            Box<Expression>,
        ),

        // Level 1: Addition & Subtraction (Happens Last)
        #[rust_sitter::prec_left(1)]
        Add(
            Box<Expression>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Sub(
            Box<Expression>,
            #[rust_sitter::leaf(text = "-")] (),
            Box<Expression>,
        ),
    }
    #[rust_sitter::extra]
    pub struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),

    }
}
