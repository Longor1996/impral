//! Parsing of tokens into Expressions and Structures.

use peekmore::PeekMore;

use super::*;

/// A expression node of an abstract syntax tree.
#[derive(Clone, PartialEq)]
pub enum Expression {
    /// A literal.
    Value(Literal),
    
    /// A structure (*not* a command!).
    Structure(Structure),
    
    /// A reference.
    Reference(ReferenceRoot),
    
    /// A command.
    Invoke(Box<Invoke>),
    
    /// A pipe.
    Pipe(Box<Pipe>),
}

/// A (data-)structure node of an abstract syntax tree.
#[derive(Clone, PartialEq)]
pub enum Structure {
    /// A list.
    List(Vec<Expression>),
    /// A dict.
    Dict(FxHashMap<CompactString, Expression>),
}

/// A reference(/variable) node of an abstract syntax tree.
#[derive(Clone, PartialEq)]
pub enum ReferenceRoot {
    /// Context Reference (`$$`)
    Ctx,
    /// Result Reference (`$`)
    Res,
    /// Local Reference (`$NAME`)
    Local(CompactString),
    /// Global Reference (`@NAME`)
    Global(CompactString),
}

/// A pipe.
#[derive(Clone, PartialEq)]
pub struct Pipe {
    /// The expression yielding pipe items.
    pub source: Expression,
    
    /// The segments of the pipe.
    pub stages: Vec<PipeSeg>,
}

/// A segment of a pipe.
#[derive(Clone, PartialEq)]
pub struct PipeSeg {
    /// If true, the invoke result is used as filter.
    pub filter: bool,
    
    /// This segments invoke.
    pub invoke: Invoke,
}

impl From<Invoke> for Expression {
    fn from(i: Invoke) -> Self {
        Self::Invoke(i.into())
    }
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::Structure(s) => std::fmt::Debug::fmt(s, f),
            Expression::Reference(r) => std::fmt::Debug::fmt(r, f),
            Expression::Invoke(c) => write!(f, "({:?})", c),
            Expression::Pipe(p) => {
                write!(f, "({:?}", p.source)?;
                for seg in &p.stages {
                    write!(f, "|")?;
                    if seg.filter {write!(f, "?")?}
                    write!(f, "{:?}", seg.invoke)?;
                }
                write!(f, ")")
            },
        }
    }
}

impl std::fmt::Debug for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(l) => std::fmt::Debug::fmt(l, f),
            Self::Dict(s) => std::fmt::Debug::fmt(s, f),
        }
    }
}

impl std::fmt::Debug for ReferenceRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ctx => write!(f, "$$"),
            Self::Res => write!(f, "$"),
            Self::Local(l) => write!(f, "${}", l),
            Self::Global(g) => write!(f, "@{}", g),
        }
    }
}

/// Parses a `TokenStream` into an AST.
pub fn parse_expression(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    first: bool
) -> Result<Expression, ParseError> {
    // Try to parse an expression item...
    let mut expr = parse_item(tokens, first)?;
    
    while let Some(token) = tokens.peek() {
        match token {
            // Dot? Subset access!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the first dot
                
                let mut get = Invoke {
                    name: "get".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                };
                
                if let Some(Token {
                    content: TokenContent::Symbol(Symbol::QuestionMark), ..
                }) = tokens.peek() {
                    get.name = "get-unwrap".into();
                    drop(tokens.next()); // drop the question-mark
                }
                
                let inner = parse_expression(tokens, false)?;
                get.pos_args.push(inner);
                
                expr = Expression::Invoke(get.into());
            },
            
            // Range? Parse Range!
            Token {
                content: TokenContent::Symbol(Symbol::Range), ..
            } => {
                drop(tokens.next()); // drop the range
                
                let mut range = Invoke {
                    name: "range".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                };
                
                let inner = parse_expression(tokens, false)?;
                range.pos_args.push(inner);
                
                expr = Expression::Invoke(range.into());
            },
            
            // QuestionMark? Existence check!
            Token {
                content: TokenContent::Symbol(Symbol::QuestionMark), ..
            } => {
                drop(tokens.next()); // drop the questionmark
                expr = Expression::Invoke(Invoke {
                    name: "exists".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Tilde? Relation!
            Token {
                content: TokenContent::Symbol(Symbol::Tilde), ..
            } => {
                drop(tokens.next()); // drop the tilde
                let to = parse_expression(tokens, false)?;
                expr = Expression::Invoke(Invoke {
                    name: "relative".into(),
                    pos_args: smallvec![expr, to],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Circle? deg2rad!
            Token {
                content: TokenContent::Symbol(Symbol::Circle), ..
            } => {
                drop(tokens.next()); // drop the circle
                expr = Expression::Invoke(Invoke {
                    name: "deg2rad".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Pipe? Pipe!
            Token {
                content: TokenContent::Symbol(Symbol::Pipe), ..
            }  if first => {
                let mut pipe = Pipe {
                    source: expr,
                    stages: vec![],
                };
                
                while let Some(Token {
                    content: TokenContent::Symbol(Symbol::Pipe), ..
                }) = tokens.peek() {
                    drop(tokens.next());
                    
                    let filter = matches!(tokens.peek(), Some(Token {content: TokenContent::Symbol(Symbol::QuestionMark),..}));
                    if filter {drop(tokens.next())}
                    
                    let invoke = parse_command(tokens, None)?;
                    
                    let seg = PipeSeg {
                        filter,
                        invoke,
                    };
                    pipe.stages.push(seg);
                }
                
                expr = Expression::Pipe(pipe.into());
            },
            
            // Ignore everything else...
            _ => break
        }
    }
    
    Ok(expr)
}

/// Parses a `TokenStream` into an item (piece of an expression).
pub fn parse_item(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    first: bool
) -> Result<Expression, ParseError> {
    // Fetch the next token...
    let token = match tokens.next() {
        // Empty case? Error!
        None => return Err(ParseError::Empty),
        Some(t) => t
    };
    
    // Remainder? Error!
    if let Token {
        content: TokenContent::Remainder(r),
        start, end: _
    } = token {
        return Err(ParseError::Unrecognized(start, r))
    };
    
    // Is it a command?
    if first {
        if let Ok(command_name) = try_into_command_name(&token) {
            return parse_command_body(command_name, tokens, None).map(|i|i.into())
        }
    }
    
    // Literal? Pass thru directly!
    if let TokenContent::Literal(l) = token.content {
        return Ok(Expression::Value(l))
    }
    
    // A group? Parse a subset!
    if let TokenContent::Group(kind, subtokens) = token.content {
        return Ok(match kind {
            Symbol::ParenLeft => parse_command(
                &mut subtokens.into_iter().peekmore(),
                Some(Symbol::ParenRight)
            ).map(|i| i.into())?,
            
            Symbol::BraketLeft => parse_list(
                &mut subtokens.into_iter().peekmore()
            ).map(|l| Expression::Structure(Structure::List(l)))?,
            
            Symbol::CurlyLeft => parse_map(
                &mut subtokens.into_iter().peekmore()
            ).map(|d| Expression::Structure(Structure::Dict(d)))?,
            _ => unreachable!()
        })
    }
    
    // A global variable?
    if let TokenContent::Symbol(Symbol::At) = token.content {
        return Ok(match tokens.next() {
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Reference(ReferenceRoot::Global(s)),
            
            Some(t) => return Err(ParseError::ExpectButGot("a global variable name".into(), format!("{}", t).into())),
            
            None => return Err(ParseError::ExpectButEnd("a global variable name")),
        })
    }
    
    // A local variable?
    if let TokenContent::Symbol(Symbol::DollarSign) = token.content {
        
        if let Some(peek) = tokens.peek().cloned() {
            // Named local variable.
            if let TokenContent::Literal(Literal::Str(s)) = peek.content {
                let _ = tokens.next();
                return Ok(Expression::Reference(ReferenceRoot::Local(s)))
            }
            
            // Numeric local variable.
            if let TokenContent::Literal(Literal::Int(i)) = peek.content {
                let _ = tokens.next();
                return Ok(Expression::Reference(ReferenceRoot::Local(i.to_string().into())))
            }
        }
        
        return Ok(Expression::Reference(ReferenceRoot::Res))
    }
    
    // A context variable?
    if let TokenContent::Symbol(Symbol::DoubleDollar) = token.content {
        return Ok(Expression::Reference(ReferenceRoot::Ctx))
    }
    
    /*
    Ok(match token {
        // Doubledot? Invalid!
        Token {
            content: TokenContent::Symbol(Symbol::DoubleDot), ..
        } => return Err(ParseError::Unexpected("double-dot in expression".into())),
        
        // Symbols means more complex parsing...
        Token {
            content: TokenContent::Symbol(s), ..
        } => {
            return Err(ParseError::Unexpected(format!("symbol in expression: {}", s).into()))
        },
        
        Token {content, ..}
        => return Err(ParseError::Unexpected(format!("token content: {:?}", content).into()))
    })
    */
    
    return Err(ParseError::Unexpected(format!("token content: {:?}", token.content).into()))
}

/// Parses the stream of tokens into a list.
pub fn parse_list(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<Vec<Expression>, ParseError> {
    let mut list = Vec::default();
    
    while let Some(t) = tokens.peek() {
        if let TokenContent::Symbol(s) = t.content {
            if s == Symbol::BraketRight {
                drop(tokens.next());
                break;
            }
            
            if s == Symbol::Comma {
                drop(tokens.next());
                continue;
            }
        }
        
        let expr = parse_expression(tokens, false)?;
        list.push(expr);
    }
    
    Ok(list)
}

/// Parses the stream of tokens into a key/value-map.
pub fn parse_map(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<FxHashMap<CompactString, Expression>, ParseError> {
    let mut map = FxHashMap::default();
    
    while let Some(t) = tokens.peek().cloned() {
        match t.content {
            TokenContent::Remainder(r )
                => return Err(ParseError::Unrecognized(t.start, r)),
            
            TokenContent::Symbol(s) => {
                if s == Symbol::CurlyRight {
                    drop(tokens.next());
                    break;
                }
                
                if s == Symbol::Comma {
                    drop(tokens.next());
                    continue;
                }
                
                return Err(ParseError::Unexpected(format!("symbol '{}'", s).into()));
            },
            TokenContent::Literal(Literal::Str(s)) => {
                let key = s;
                drop(tokens.next()); // eat key
                
                let next = tokens.next();
                if let Some(Token {content: TokenContent::Symbol(Symbol::EqualSign), ..}) = next {
                    // everything checks out, continue on...
                } else {
                    return Err(ParseError::Unexpected("token".into()));
                }
                
                let expr = parse_expression(tokens, false)?;
                map.insert(key, expr);
            },
            TokenContent::Literal(l) => return Err(ParseError::Unexpected(format!("literal {:?}", l).into())),
            g @ TokenContent::Group(_, _) => return Err(ParseError::Unexpected(format!("group {:?}", g).into()))
        };
    }
    
    Ok(map)
}
