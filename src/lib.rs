pub mod interpreter;
pub mod compiler;
pub mod build_manager;
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
        On {
            #[rust_sitter::leaf(text = "on")] _on: (),
            #[rust_sitter::leaf(text = "(")] _l: (),
            condition: Expression,
            #[rust_sitter::leaf(text = ")")] _r: (),
            body: Block,
            // The 'off' part is optional (Option<Box<...>>)
            else_clause: Option<OffClause>, 
        },
        LoopOn {
            #[rust_sitter::leaf(text = "loop")] _loop: (),
            #[rust_sitter::leaf(text = "on")] _on: (),
            #[rust_sitter::leaf(text = "(")] _l: (),
            condition: Expression,
            #[rust_sitter::leaf(text = ")")] _r: (),
            body: Block,
        },

        // 4. The "For" Loop: loop x in y [per z] [on (cond)] { } [off { }]
        LoopIter {
            #[rust_sitter::leaf(text = "loop")] _loop: (),
            #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
            iterator: String,
            #[rust_sitter::leaf(text = "in")] _in: (),
            iterable: Expression, // This handles 'arr' or '0..10'
            
            step: Option<StepClause>,   // Optional "per 5"
            filter: Option<LoopFilter>, // Optional "on (x % 2 == 0)"
            
            body: Block,
            
            // Optional "off" block for the filter
            else_clause: Option<OffClause>,
        },
        Print(
            #[rust_sitter::leaf(text = "print")] (),
            Expression
        )
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
        #[rust_sitter::prec_left(0)]
        Eq(
            Box<Expression>, 
            #[rust_sitter::leaf(text = "==")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)]
        Neq(
            Box<Expression>, 
            #[rust_sitter::leaf(text = "!=")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)]
        Gt(
            Box<Expression>, 
            #[rust_sitter::leaf(text = ">")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)]
        Lt(
            Box<Expression>, 
            #[rust_sitter::leaf(text = "<")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)]
        Geq(
            Box<Expression>, 
            #[rust_sitter::leaf(text = ">=")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)]
        Leq(
            Box<Expression>, 
            #[rust_sitter::leaf(text = "<=")] (), 
            Box<Expression>
        ),
        #[rust_sitter::prec_left(0)] // Low priority
        Range(
            Box<Expression>, 
            #[rust_sitter::leaf(text = "..")] (), 
            Box<Expression>
        ),
    }
    #[rust_sitter::extra]
    pub struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),

    }
    pub struct Block {
        #[rust_sitter::leaf(text = "{")] _l: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")] _r: (),
    }
    pub struct OffClause {
        #[rust_sitter::leaf(text = "off")] _off: (),
        pub body: Block,
    }
    pub struct StepClause {
        #[rust_sitter::leaf(text = "per")] _per: (),
        pub value: Expression,
    }
    pub struct LoopFilter {
        #[rust_sitter::leaf(text = "on")] _on: (),
        #[rust_sitter::leaf(text = "(")] _l: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ")")] _r: (),
    }
}
