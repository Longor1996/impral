//! Parsing of tokens into Expressions and Structures.

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
    
    // TODO: Make pipes a language structure!
    // /// A pipe.
    // Pipe(Box<Invoke>),
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
    /// Context Reference (`$`)
    Ctx,
    /// Result Reference (`$$`)
    Res,
    /// Local Reference (`$NAME`)
    Local(CompactString),
    /// Global Reference (`@NAME`)
    Global(CompactString),
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
            Expression::Reference(ReferenceRoot::Ctx) => write!(f, "$"),
            Expression::Reference(ReferenceRoot::Res) => write!(f, "$$"),
            Expression::Reference(ReferenceRoot::Local(l)) => write!(f, "${}", l),
            Expression::Reference(ReferenceRoot::Global(g)) => write!(f, "@{}", g),
            Expression::Invoke(c) => write!(f, "({:?})", c),
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
            Self::Ctx => write!(f, "$"),
            Self::Res => write!(f, "$$"),
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
    
    if let Some(token) = tokens.peek() {
        match token {
            // Dot? Subset access!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the dot
                
                if let Some(Token {
                    content: TokenContent::Symbol(Symbol::Dot), ..
                }) = tokens.peek() {
                    drop(tokens.next()); // drop the second dot
                    
                    let mut range = Invoke {
                        name: "range".into(),
                        pos_args: smallvec![expr],
                        nom_args: Default::default(),
                    };
                    
                    let inner = parse_expression(tokens, false)?;
                    range.pos_args.push(inner);
                    
                    expr = Expression::Invoke(range.into());
                }
                else {
                    let mut get = Invoke {
                        name: "idx".into(),
                        pos_args: smallvec![expr],
                        nom_args: Default::default(),
                    };
                    
                    if let Some(Token {
                        content: TokenContent::Symbol(Symbol::QuestionMark), ..
                    }) = tokens.peek() {
                        get.name = "idxn".into();
                        drop(tokens.next()); // drop the question-mark
                    }
                    
                    let inner = parse_expression(tokens, false)?;
                    get.pos_args.push(inner);
                    
                    expr = Expression::Invoke(get.into());
                }
            },
            
            // QuestionMark? Existence check!
            Token {
                content: TokenContent::Symbol(Symbol::QuestionMark), ..
            } => {
                drop(tokens.next()); // drop the dot
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
                drop(tokens.next()); // drop the dot
                let to = parse_expression(tokens, false)?;
                expr = Expression::Invoke(Invoke {
                    name: "rel".into(),
                    pos_args: smallvec![expr, to],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Ignore everything else...
            _ => ()
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
        if let Ok(command_name) = token.try_into_command_name() {
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
                &mut subtokens.into_iter().peekable(),
                Some(Symbol::ParenRight)
            ).map(|i| i.into())?,
            
            Symbol::BraketLeft => parse_list(
                &mut subtokens.into_iter().peekable()
            ).map(|l| Expression::Structure(Structure::List(l)))?,
            
            Symbol::CurlyLeft => parse_map(
                &mut subtokens.into_iter().peekable()
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
        return Ok(Expression::Reference(ReferenceRoot::Res))
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
