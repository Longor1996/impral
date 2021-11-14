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
pub fn parse_expression(tokens: &mut PeekableTokenStream<impl TokenStream>) -> Result<Expression, ParseError> {
    let mut expr = match tokens.next() {
        // Empty case? Error!
        None => return Err(ParseError::Empty),
        
        // Remainder? Error!
        Some(Token {
            content: TokenContent::Remainder(r),
            position
        }) => return Err(ParseError::Unrecognized(position, r)),
        
        // Literal? Pass thru directly!
        Some(Token {
            content: TokenContent::Literal(l), ..
        }) => Expression::Value(l),
        
        // Parentheses? Parse a command!
        Some(Token {
            content: TokenContent::Symbol(Symbol::ParenLeft), ..
        }) => parse_command(tokens, Some(Symbol::ParenRight)).map(|i| i.into())?,
        
        // Brackets? Parse a list!
        Some(Token {
            content: TokenContent::Symbol(Symbol::BraketLeft), ..
        }) => parse_list(tokens).map(|v| Expression::Structure(Structure::List(v)))?,
        
        // Curlies? Parse a map!
        Some(Token {
            content: TokenContent::Symbol(Symbol::CurlyLeft), ..
        }) => parse_map(tokens).map(|d| Expression::Structure(Structure::Dict(d)))?,
        
        // Global Variable!
        Some(Token {
            content: TokenContent::Symbol(Symbol::At), ..
        }) => match tokens.next() {
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Reference(ReferenceRoot::Global(s)),
            Some(t) => return Err(ParseError::ExpectButGot("a global variable name".into(), format!("{}", t).into())),
            None => return Err(ParseError::ExpectButEnd("a global variable name")),
        },
        
        // Local Variable!
        Some(Token {
            content: TokenContent::Symbol(Symbol::DollarSign), ..
        }) => match tokens.next() {
            Some(Token {
                content: TokenContent::Symbol(Symbol::DollarSign), ..
            }) => Expression::Reference(ReferenceRoot::Res),
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Reference(ReferenceRoot::Local(s)),
            Some(Token {
                content: TokenContent::Literal(Literal::Int(i)), ..
            }) => Expression::Reference(ReferenceRoot::Local(i.to_string().into())),
            Some(t) => return Err(ParseError::ExpectButGot("a local variable name".into(), format!("{}", t).into())),
            None => return Err(ParseError::ExpectButEnd("a local variable name")),
        },
        
        // Doubledot? Invalid!
        Some(Token {
            content: TokenContent::Symbol(Symbol::DoubleDot), ..
        }) => return Err(ParseError::Unexpected("double-dot in expression".into())),
        
        // Symbols means more complex parsing...
        Some(Token {
            content: TokenContent::Symbol(s), ..
        }) => {
            return Err(ParseError::Unexpected(format!("symbol in expression: {}", s).into()))
        }
    };
    
    if let Some(token) = tokens.peek() {
        match token {
            // Dot? Subset access!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the dot
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
                
                let inner = parse_expression(tokens)?;
                get.pos_args.push(inner);
                
                expr = Expression::Invoke(get.into());
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
                let to = parse_expression(tokens)?;
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

/// Parses the stream of tokens into a list.
pub fn parse_list(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<Vec<Expression>, ParseError> {
    let mut list = Vec::default();
    
    loop {
        match tokens.peek().cloned() {
            Some(t) => {
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
                
                let expr = parse_expression(tokens)?;
                list.push(expr);
            },
            None => return Err(ParseError::ExpectButEnd("end of list ']'")),
        }
    }
    
    Ok(list)
}

/// Parses the stream of tokens into a key/value-map.
pub fn parse_map(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<FxHashMap<CompactString, Expression>, ParseError> {
    let mut map = FxHashMap::default();
    
    loop {
        match tokens.peek().cloned() {
            Some(t) => {
                match t.content {
                    TokenContent::Remainder(r )
                        => return Err(ParseError::Unrecognized(t.position, r)),
                    
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
                        
                        let expr = parse_expression(tokens)?;
                        map.insert(key, expr);
                    },
                    TokenContent::Literal(l) => return Err(ParseError::Unexpected(format!("literal {:?}", l).into()))
                };
            },
            None => return Err(ParseError::ExpectButEnd("end of list ']'")),
        }
    }
    
    Ok(map)
}
