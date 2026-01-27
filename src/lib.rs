pub mod build_manager;
pub mod compiler;
pub mod interpreter;
#[rust_sitter::grammar("kiro")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub statements: Vec<Statement>,
    }
    // 1. The Wrapper Struct
    #[derive(Debug, Clone)]
    pub struct NumberVal {
        #[rust_sitter::leaf(pattern = r"\d+(\.\d+)?", transform = |s| s.to_string())]
        pub value: String,
    }
    #[derive(Debug, Clone)]
    pub struct VariableVal {
        #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
        pub value: String,
    }

    // 2. Wrapper for String Literals ("hello")
    #[derive(Debug, Clone)]
    pub struct StringVal {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |s| s.to_string())]
        pub value: String,
    }
    #[derive(Debug, Clone)]
    pub enum BoolVal {
        True(#[rust_sitter::leaf(text = "true")] ()),
        False(#[rust_sitter::leaf(text = "false")] ()),
    }
    #[derive(Debug, Clone)]
    pub enum KiroType {
        #[rust_sitter::leaf(text = "num")]
        Num, // Replaces Int
        #[rust_sitter::leaf(text = "str")]
        Str, // New
        #[rust_sitter::leaf(text = "bool")]
        Bool, // New
        #[rust_sitter::leaf(text = "adr")]
        Adr,
        #[rust_sitter::leaf(text = "pipe")] Pipe,
    }
    #[derive(Debug, Clone)]
    pub struct FuncParam {
        #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
        pub name: String,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub command_type: KiroType,
    }
    #[derive(Debug, Clone)]
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
            #[rust_sitter::leaf(text = "on")]
            _on: (),
            #[rust_sitter::leaf(text = "(")]
            _l: (),
            condition: Expression,
            #[rust_sitter::leaf(text = ")")]
            _r: (),
            body: Block,
            // The 'off' part is optional (Option<Box<...>>)
            else_clause: Option<OffClause>,
        },
        LoopOn {
            #[rust_sitter::leaf(text = "loop")]
            _loop: (),
            #[rust_sitter::leaf(text = "on")]
            _on: (),
            #[rust_sitter::leaf(text = "(")]
            _l: (),
            condition: Expression,
            #[rust_sitter::leaf(text = ")")]
            _r: (),
            body: Block,
        },

        // 4. The "For" Loop: loop x in y [per z] [on (cond)] { } [off { }]
        LoopIter {
            #[rust_sitter::leaf(text = "loop")]
            _loop: (),
            #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
            iterator: String,
            #[rust_sitter::leaf(text = "in")]
            _in: (),
            iterable: Expression, // This handles 'arr' or '0..10'

            step: Option<StepClause>,   // Optional "per 5"
            filter: Option<LoopFilter>, // Optional "on (x % 2 == 0)"

            body: Block,

            // Optional "off" block for the filter
            else_clause: Option<OffClause>,
        },
        FunctionDef {
            #[rust_sitter::leaf(text = "pure")]
            pure_kw: Option<()>, // Optional "pure" keyword

            #[rust_sitter::leaf(text = "fn")]
            _fn: (),

            #[rust_sitter::leaf(pattern = r"[a-z_]+", transform = |s| s.to_string())]
            name: String,

            #[rust_sitter::leaf(text = "(")]
            _l: (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")] ()
            )]
            params: Vec<FuncParam>,
            #[rust_sitter::leaf(text = ")")]
            _r: (),

            body: Block,
        },
        // 1. Give: give <channel> <value>
        Give(
            #[rust_sitter::leaf(text = "give")] (),
            Expression, // Channel
            Expression, // Value
        ),

        // 2. Close: close <channel>
        Close(
            #[rust_sitter::leaf(text = "close")] (),
            Expression, // Channel
        ),
        ExprStmt(Expression),
        Print(#[rust_sitter::leaf(text = "print")] (), Expression),
    }
    #[derive(Debug, Clone)]
    pub enum Expression {
        // 2. New Literals
        #[rust_sitter::prec_left(1)]
        BoolLit(BoolVal),

        #[rust_sitter::prec_left(1)]
        Number(NumberVal),

        #[rust_sitter::prec_left(1)]
        StringLit(StringVal),

        #[rust_sitter::prec_left(1)]
        Variable(VariableVal),

        #[rust_sitter::prec_left(1)]
        PipeInit(
            #[rust_sitter::leaf(text = "pipe")] (),
            KiroType, // The type of data in the pipe
        ),

        // 4. Take: take <channel>
        // Example: var x = take p
        #[rust_sitter::prec_right(4)]
        Take(
            #[rust_sitter::leaf(text = "take")] (),
            Box<Expression>,
        ),
        // 3. Pointer Logic
        // ref x
        #[rust_sitter::prec_right(4)] // Right-associative
        Ref(#[rust_sitter::leaf(text = "ref")] (), Box<Expression>),

        // deref x
        #[rust_sitter::prec_right(4)]
        Deref(#[rust_sitter::leaf(text = "deref")] (), Box<Expression>),
        #[rust_sitter::prec_left(3)] // High precedence
        Call(
            Box<Expression>, // The function name (usually a Variable)
            #[rust_sitter::leaf(text = "(")] (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")] ()
            )]
            Vec<Expression>, // Arguments
            #[rust_sitter::leaf(text = ")")] (),
        ),

        // 5. Async "Run" Call
        // Syntax: run foo(1, 2)
        #[rust_sitter::prec_left(2)]
        RunCall(
            #[rust_sitter::leaf(text = "run")] (),
            Box<Expression>, // Should be a Call expression
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
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Neq(
            Box<Expression>,
            #[rust_sitter::leaf(text = "!=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Gt(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Lt(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Geq(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Leq(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)] // Low priority
        Range(
            Box<Expression>,
            #[rust_sitter::leaf(text = "..")] (),
            Box<Expression>,
        ),
    }
    #[rust_sitter::extra]
    pub struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+|//[^\n]*")]
        _whitespace: (),
    }
    #[derive(Debug, Clone)]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _l: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _r: (),
    }
    #[derive(Debug, Clone)]
    pub struct OffClause {
        #[rust_sitter::leaf(text = "off")]
        _off: (),
        pub body: Block,
    }
    #[derive(Debug, Clone)]
    pub struct StepClause {
        #[rust_sitter::leaf(text = "per")]
        _per: (),
        pub value: Expression,
    }
    #[derive(Debug, Clone)]
    pub struct LoopFilter {
        #[rust_sitter::leaf(text = "on")]
        _on: (),
        #[rust_sitter::leaf(text = "(")]
        _l: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ")")]
        _r: (),
    }
}
