// Production-quality recursive descent parser for Axiom — Final Maturation
//
// FIXES IN THIS VERSION:
//   1. parse_out_stmt — space-separated args: `out "text" var "text"` now works.
//      Previously only comma-separated args were collected; everything after the
//      first arg was silently dropped if no comma followed.
//   2. is_match_body_ahead — walks past Ident.Ident dotted paths before =>
//   3. parse_match_pattern — handles Status.Active dotted-path patterns
//   4. parse() — hoists ALL declarations before ALL statements so the runtime
//      always sees functions/classes/enums registered before executing calls.

use crate::ast::{
    ClassMember, EnumVariant, Expr, Item, MatchArm, MatchPattern, Stmt, StringPart,
};
use crate::errors::{ParserError, Span};
use crate::lexer::{Lexer, Token};
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<(Token, Span)>,
    source_id: u32,
}

impl Parser {
    pub fn new(source: &str, source_id: u32) -> Self {
        let mut lexer = Lexer::new(source, source_id);
        let tokens = lexer.tokenize();
        Parser {
            tokens: VecDeque::from(tokens),
            source_id,
        }
    }

    // -----------------------------------------------------------------------
    // Public entry point
    //
    // Hoists ALL declarations before ALL statements so the runtime always finds
    // every function/class/enum registered before executing any call.
    // -----------------------------------------------------------------------
    pub fn parse(&mut self) -> Result<Vec<Item>, ParserError> {
        let mut decls: Vec<Item> = Vec::new();
        let mut stmts: Vec<Item> = Vec::new();

        while !self.is_at_end() {
            self.skip_semicolons();
            if self.is_at_end() { break; }
            let item = self.parse_item()?;
            match &item {
                Item::FunctionDecl { .. }
                | Item::ClassDecl { .. }
                | Item::EnumDecl { .. }
                | Item::StdImport { .. }
                | Item::LocImport { .. }
                | Item::LibDecl { .. } => decls.push(item),
                Item::Statement(_) => stmts.push(item),
            }
        }

        decls.extend(stmts);
        Ok(decls)
    }

    // -----------------------------------------------------------------------
    // Items
    // -----------------------------------------------------------------------

    fn parse_item(&mut self) -> Result<Item, ParserError> {
        self.skip_semicolons();
        match self.peek_token() {
            Token::Fun => self.parse_function(),
            Token::Cls => self.parse_class_decl(),
            Token::Enm => self.parse_enum_decl(),
            Token::Std => self.parse_std_import(),
            Token::Loc => self.parse_loc_import(),
            Token::Lib => self.parse_lib_decl(),
            Token::Ident(_) => match self.peek_nth(1) {
                Token::LParen => {
                    if self.is_func_decl_ahead() {
                        self.parse_function()
                    } else {
                        let stmt = self.parse_stmt()?;
                        Ok(Item::Statement(stmt))
                    }
                }
                Token::LBrace => self.parse_type_decl(),
                _ => {
                    let stmt = self.parse_stmt()?;
                    Ok(Item::Statement(stmt))
                }
            },
            _ => {
                let stmt = self.parse_stmt()?;
                Ok(Item::Statement(stmt))
            }
        }
    }

    // name ( params ) {   → true  (function declaration)
    // name ( args   )     → false (call expression)
    fn is_func_decl_ahead(&self) -> bool {
        if !matches!(self.peek_token(), Token::Ident(_)) { return false; }
        let mut i = 1usize;
        if !matches!(self.peek_nth(i), Token::LParen) { return false; }
        let mut depth = 0usize;
        while i < self.tokens.len() {
            match &self.tokens[i].0 {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1 < self.tokens.len()
                            && matches!(&self.tokens[i + 1].0, Token::LBrace);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    // Genesis-style: Name { … } — decide class vs enum by peeking inside
    fn parse_type_decl(&mut self) -> Result<Item, ParserError> {
        let mut is_enum = true;
        let mut i = 2; // skip name(0) and '{'(1)
        while i < self.tokens.len() && !matches!(&self.tokens[i].0, Token::RBrace | Token::Eof) {
            match &self.tokens[i].0 {
                Token::Fun | Token::Let => { is_enum = false; break; }
                Token::Ident(_) => {
                    if i + 1 < self.tokens.len()
                        && matches!(&self.tokens[i + 1].0, Token::LParen)
                    {
                        is_enum = false;
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if is_enum { self.parse_enum_decl_internal(false) } else { self.parse_class_decl_internal(false) }
    }

    fn parse_function(&mut self) -> Result<Item, ParserError> {
        let start = self.current_span();
        if matches!(self.peek_token(), Token::Fun) { self.advance(); }
        let name   = self.consume_ident()?;
        self.consume(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.consume(Token::RParen)?;
        let body   = self.parse_block()?;
        Ok(Item::FunctionDecl { name, params, body, span: start.merge(self.prev_span()) })
    }

    fn parse_class_decl(&mut self) -> Result<Item, ParserError> {
        self.parse_class_decl_internal(true)
    }

    fn parse_class_decl_internal(&mut self, consume_kw: bool) -> Result<Item, ParserError> {
        let start = self.current_span();
        if consume_kw { self.advance(); } // 'cls'
        let name = self.consume_ident()?;
        let parent = if matches!(self.peek_token(), Token::Ext) {
            self.advance();
            Some(self.consume_ident()?)
        } else {
            None
        };
        self.consume(Token::LBrace)?;
        let mut body = Vec::new();

        while !matches!(self.peek_token(), Token::RBrace | Token::Eof) {
            self.skip_semicolons();
            if matches!(self.peek_token(), Token::RBrace) { break; }

            if matches!(self.peek_token(), Token::Fun | Token::Ident(_)) {
                let mstart = self.current_span();
                if matches!(self.peek_token(), Token::Fun) { self.advance(); }
                if matches!(self.peek_nth(1), Token::LParen) {
                    let method_name = self.consume_ident()?;
                    self.consume(Token::LParen)?;
                    let params = self.parse_param_list()?;
                    self.consume(Token::RParen)?;
                    let mbody = self.parse_block()?;
                    body.push(ClassMember::Method {
                        name: method_name, params, body: mbody,
                        span: mstart.merge(self.prev_span()),
                    });
                } else {
                    return Err(ParserError::InvalidSyntax {
                        context: "class body: expected method name followed by '('".to_string(),
                        span: self.current_span(),
                    });
                }
            } else if matches!(self.peek_token(), Token::Let) {
                let fstart = self.current_span();
                self.advance();
                let field_name = self.consume_ident()?;
                let default = if matches!(self.peek_token(), Token::Assign) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else { None };
                self.skip_semicolons();
                body.push(ClassMember::Field {
                    name: field_name, default, span: fstart.merge(self.prev_span()),
                });
            } else {
                return Err(ParserError::InvalidSyntax {
                    context: "class body (expected 'fun', 'let', or method name)".to_string(),
                    span: self.current_span(),
                });
            }
        }
        self.consume(Token::RBrace)?;
        Ok(Item::ClassDecl { name, parent, body, span: start.merge(self.prev_span()) })
    }

    fn parse_enum_decl(&mut self) -> Result<Item, ParserError> {
        self.parse_enum_decl_internal(true)
    }

    fn parse_enum_decl_internal(&mut self, consume_kw: bool) -> Result<Item, ParserError> {
        let start = self.current_span();
        if consume_kw { self.advance(); }
        let name = self.consume_ident()?;
        self.consume(Token::LBrace)?;
        let mut variants = Vec::new();

        while !matches!(self.peek_token(), Token::RBrace | Token::Eof) {
            self.skip_semicolons();
            if matches!(self.peek_token(), Token::RBrace) { break; }
            let vstart = self.current_span();
            let vname  = self.consume_ident()?;
            let has_data = if matches!(self.peek_token(), Token::LParen) {
                self.advance();
                while !matches!(self.peek_token(), Token::RParen | Token::Eof) { self.advance(); }
                self.consume(Token::RParen)?;
                true
            } else { false };
            variants.push(EnumVariant { name: vname, has_data, span: vstart.merge(self.prev_span()) });
            if matches!(self.peek_token(), Token::Comma) { self.advance(); }
        }
        self.consume(Token::RBrace)?;
        Ok(Item::EnumDecl { name, variants, span: start.merge(self.prev_span()) })
    }

    fn parse_std_import(&mut self) -> Result<Item, ParserError> {
        let start = self.current_span();
        self.advance();
        let name = self.consume_ident()?;
        self.skip_semicolons();
        Ok(Item::StdImport { name, span: start.merge(self.prev_span()) })
    }

    fn parse_loc_import(&mut self) -> Result<Item, ParserError> {
        let start = self.current_span();
        self.advance();
        let name = self.consume_ident()?;
        self.skip_semicolons();
        Ok(Item::LocImport { name, span: start.merge(self.prev_span()) })
    }

    fn parse_lib_decl(&mut self) -> Result<Item, ParserError> {
        let start = self.current_span();
        self.advance();
        let name = self.consume_ident()?;
        Ok(Item::LibDecl { name, span: start.merge(self.prev_span()) })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, ParserError> {
        let mut params = Vec::new();
        if matches!(self.peek_token(), Token::RParen | Token::Eof) { return Ok(params); }
        loop {
            if matches!(self.peek_token(), Token::SelfKw) {
                self.advance();
                params.push("self".to_string());
            } else {
                params.push(self.consume_ident()?);
            }
            if !matches!(self.peek_token(), Token::Comma) { break; }
            self.advance();
        }
        Ok(params)
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.skip_semicolons();
        match self.peek_token() {
            Token::Let    => self.parse_let_stmt(),
            Token::If     => self.parse_if_stmt(),
            Token::While  => self.parse_while_stmt(),
            Token::For    => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Go     => self.parse_go_stmt(),
            Token::Match  => self.parse_match_stmt(),
            Token::Out    => self.parse_out_stmt(),
            Token::LBrace => { let b = self.parse_block()?; Ok(Stmt::Block(b)) }
            _ => {
                if matches!(self.peek_token(), Token::RBrace | Token::Eof) {
                    return Err(ParserError::UnexpectedToken {
                        expected: "statement".to_string(),
                        found: format!("{:?} in parse_stmt", self.peek_token()),
                        span: self.current_span(),
                    });
                }
                let expr = self.parse_expr()?;
                if matches!(self.peek_token(), Token::Semicolon) { self.advance(); }
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span();
        self.advance();
        let name  = self.consume_ident()?;
        self.consume(Token::Assign)?;
        let value = self.parse_expr()?;
        self.skip_semicolons();
        Ok(Stmt::Let { name, value, span: start.merge(self.prev_span()) })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expr()?;

        if self.is_match_body_ahead() {
            return self.parse_if_as_match(condition, start);
        }

        let then_body = self.parse_block()?;
        let else_body = if matches!(self.peek_token(), Token::Else) {
            self.advance();
            if matches!(self.peek_token(), Token::If) {
                Some(vec![self.parse_if_stmt()?])
            } else {
                Some(self.parse_block()?)
            }
        } else { None };

        Ok(Stmt::If { condition, then_body, else_body, span: start.merge(self.prev_span()) })
    }

    // -----------------------------------------------------------------------
    // is_match_body_ahead — peeks inside { … } without consuming.
    //
    // Handles all pattern forms:
    //   Ident =>                    bare variant
    //   Ident.Ident =>              dotted path (Status.Active)
    //   Ident(...) =>               variant with binding
    //   Number|String|True|False => literal
    //   els =>                      wildcard
    // -----------------------------------------------------------------------
    fn is_match_body_ahead(&self) -> bool {
        if !matches!(self.peek_token(), Token::LBrace) { return false; }
        let mut i = 1usize;
        while i < self.tokens.len() && matches!(&self.tokens[i].0, Token::Semicolon) { i += 1; }
        if i >= self.tokens.len() { return false; }

        match &self.tokens[i].0 {
            Token::Ident(_) | Token::Number(_) | Token::String(_)
            | Token::True | Token::False | Token::Els => {}
            _ => return false,
        }

        loop {
            i += 1;
            if i >= self.tokens.len() { return false; }
            match &self.tokens[i].0 {
                Token::Dot => {
                    i += 1;
                    if i >= self.tokens.len() { return false; }
                    if !matches!(&self.tokens[i].0, Token::Ident(_)) { return false; }
                }
                Token::LParen => {
                    let mut depth = 0usize;
                    while i < self.tokens.len() {
                        match &self.tokens[i].0 {
                            Token::LParen  => depth += 1,
                            Token::RParen  => { depth -= 1; if depth == 0 { i += 1; break; } }
                            Token::Eof     => return false,
                            _ => {}
                        }
                        i += 1;
                    }
                    return i < self.tokens.len() && matches!(&self.tokens[i].0, Token::Arrow);
                }
                Token::Arrow => return true,
                _ => return false,
            }
        }
    }

    fn parse_if_as_match(&mut self, expr: Expr, start: Span) -> Result<Stmt, ParserError> {
        self.consume(Token::LBrace)?;
        let mut arms = Vec::new();

        while !matches!(self.peek_token(), Token::RBrace | Token::Eof) {
            self.skip_semicolons();
            if matches!(self.peek_token(), Token::RBrace) { break; }

            let astart  = self.current_span();
            let pattern = self.parse_match_pattern()?;
            self.consume(Token::Arrow)?;

            let body = if matches!(self.peek_token(), Token::LBrace) {
                self.parse_block()?
            } else {
                vec![self.parse_stmt()?]
            };

            self.skip_semicolons();
            if matches!(self.peek_token(), Token::Comma) { self.advance(); }

            arms.push(MatchArm { pattern, body, span: astart.merge(self.prev_span()) });
        }

        self.consume(Token::RBrace)?;
        Ok(Stmt::Match { expr, arms, span: start.merge(self.prev_span()) })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span(); self.advance();
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body, span: start.merge(self.prev_span()) })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span(); self.advance();
        let var = self.consume_ident()?;
        self.consume(Token::In)?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iterable, body, span: start.merge(self.prev_span()) })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span(); self.advance();
        let value = if matches!(self.peek_token(), Token::Semicolon | Token::RBrace | Token::Eof) {
            None
        } else { Some(self.parse_expr()?) };
        self.skip_semicolons();
        Ok(Stmt::Return { value, span: start.merge(self.prev_span()) })
    }

    fn parse_go_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span(); self.advance();
        let body = self.parse_block()?;
        Ok(Stmt::GoSpawn { body, span: start.merge(self.prev_span()) })
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span(); self.advance();
        let expr = self.parse_expr()?;
        self.parse_if_as_match(expr, start)
    }

    // -----------------------------------------------------------------------
    // parse_match_pattern
    // -----------------------------------------------------------------------
    fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParserError> {
        match self.peek_token() {
            Token::Els => { self.advance(); Ok(MatchPattern::Wildcard) }

            Token::Ident(name) => {
                if name == "_" { self.advance(); return Ok(MatchPattern::Wildcard); }
                self.advance();

                if matches!(self.peek_token(), Token::Dot) {
                    self.advance();
                    let variant = self.consume_ident()?;
                    let binding = if matches!(self.peek_token(), Token::LParen) {
                        self.advance();
                        let b = if matches!(self.peek_token(), Token::RParen) { None }
                                else { Some(self.consume_ident()?) };
                        self.consume(Token::RParen)?;
                        b
                    } else { None };
                    return Ok(MatchPattern::EnumVariant { enum_name: Some(name), variant, binding });
                }

                if matches!(self.peek_token(), Token::LParen) {
                    self.advance();
                    let binding = if matches!(self.peek_token(), Token::RParen) { None }
                                  else { Some(self.consume_ident()?) };
                    self.consume(Token::RParen)?;
                    return Ok(MatchPattern::EnumVariant { enum_name: None, variant: name, binding });
                }

                Ok(MatchPattern::Identifier(name))
            }

            Token::Number(_) | Token::String(_) | Token::True | Token::False => {
                let expr = self.parse_primary()?;
                Ok(MatchPattern::Literal(expr))
            }

            tok => Err(ParserError::UnexpectedToken {
                expected: "match pattern".to_string(),
                found: format!("{:?} (next: {:?})", tok, self.peek_nth(1)),
                span: self.current_span(),
            }),
        }
    }

    // -----------------------------------------------------------------------
    // parse_out_stmt — THE KEY FIX
    //
    // Axiom's `out` statement accepts SPACE-SEPARATED arguments:
    //   out "text" var "text";         ← 3 args, no commas
    //   out "fib(" i ") = " fib(i);   ← 4 args, no commas
    //
    // The previous implementation only looped on commas, so only the first
    // argument was ever collected.
    //
    // Fix: after each expression, if the next token can START another
    // expression (and we're not at a statement boundary or match-arm
    // separator), continue parsing without requiring a comma.
    //
    // Commas are still accepted as optional separators (and we still detect
    // match-arm comma boundaries correctly).
    // -----------------------------------------------------------------------
    fn parse_out_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start = self.current_span();
        self.advance(); // consume 'out'

        let mut arguments = Vec::new();

        // Keep collecting expressions as long as the next token can start one
        // and we haven't hit a hard boundary.
        while self.token_can_start_expr(&self.peek_token()) {
            arguments.push(self.parse_expr()?);

            // After each expr, check for an optional comma separator
            if matches!(self.peek_token(), Token::Comma) {
                // Comma followed by a match-arm token → this comma belongs to
                // the match arm list, not to us. Stop here.
                if matches!(self.peek_nth(1),
                    Token::Arrow | Token::RParen | Token::RBrace
                    | Token::RBracket | Token::Els)
                {
                    break;
                }
                // comma then Ident => Arrow  (bare arm)
                if matches!(self.peek_nth(2), Token::Arrow) { break; }
                // comma then Ident.Ident =>  (dotted arm)
                if matches!(self.peek_nth(1), Token::Ident(_))
                    && matches!(self.peek_nth(2), Token::Dot)
                {
                    break;
                }
                // Safe to consume the comma as a separator and keep going
                self.advance();
            }
            // No comma: if next token can start an expr, loop naturally;
            // otherwise the while condition will stop us.
        }

        self.skip_semicolons();
        Ok(Stmt::Out { arguments, span: start.merge(self.prev_span()) })
    }

    // -----------------------------------------------------------------------
    // token_can_start_expr
    //
    // Returns true if `tok` is a valid first token of an expression.
    // Statement-opening keywords are deliberately excluded so that:
    //   out "done"
    //   out "next line"        ← this should NOT be consumed as a second arg
    //   if x { ... }          ← this should NOT be consumed as a third arg
    // -----------------------------------------------------------------------
    fn token_can_start_expr(&self, tok: &Token) -> bool {
        matches!(tok,
            Token::Ident(_)
            | Token::Number(_)
            | Token::String(_)
            | Token::InterpolatedString(_)
            | Token::True
            | Token::False
            | Token::LParen
            | Token::LBracket
            | Token::SelfKw
            | Token::New
            | Token::Dot       // .member shorthand
            | Token::Minus     // unary minus
            | Token::Not       // unary !
        )
        // Excluded (statement keywords / hard boundaries):
        //   Out, Let, If, Else, While, For, Return, Match, Go,
        //   Fun, Cls, Enm, Std, Loc, Lib, New (already included above),
        //   RBrace, RBracket, RParen, Comma, Semicolon, Arrow, Eof
    }

    // -----------------------------------------------------------------------
    // Block
    // -----------------------------------------------------------------------

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        self.consume(Token::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek_token(), Token::RBrace | Token::Eof) {
            self.skip_semicolons();
            if matches!(self.peek_token(), Token::RBrace) { break; }
            stmts.push(self.parse_stmt()?);
        }
        self.consume(Token::RBrace)?;
        Ok(stmts)
    }

    // -----------------------------------------------------------------------
    // Expressions
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self) -> Result<Expr, ParserError> { self.parse_assignment() }

    fn parse_assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.parse_logical_or()?;
        if matches!(self.peek_token(), Token::Assign) {
            let start = expr.span(); self.advance();
            let value = self.parse_assignment()?;
            let span  = start.merge(value.span());
            return Ok(Expr::Assign { target: Box::new(expr), value: Box::new(value), span });
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_logical_and()?;
        while matches!(self.peek_token(), Token::Or) {
            let start = expr.span(); self.advance();
            let right = self.parse_logical_and()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: "||".into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_equality()?;
        while matches!(self.peek_token(), Token::And) {
            let start = expr.span(); self.advance();
            let right = self.parse_equality()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: "&&".into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_comparison()?;
        while matches!(self.peek_token(), Token::Equal | Token::NotEqual) {
            let op = if matches!(self.peek_token(), Token::Equal) { "==" } else { "!=" };
            let start = expr.span(); self.advance();
            let right = self.parse_comparison()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: op.into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_term()?;
        while let Some(op) = match self.peek_token() {
            Token::Less         => Some("<"),
            Token::LessEqual    => Some("<="),
            Token::Greater      => Some(">"),
            Token::GreaterEqual => Some(">="),
            _                   => None,
        } {
            let start = expr.span(); self.advance();
            let right = self.parse_term()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: op.into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_factor()?;
        while let Some(op) = match self.peek_token() {
            Token::Plus  => Some("+"),
            Token::Minus => Some("-"),
            _            => None,
        } {
            let start = expr.span(); self.advance();
            let right = self.parse_factor()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: op.into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_unary()?;
        while let Some(op) = match self.peek_token() {
            Token::Star    => Some("*"),
            Token::Slash   => Some("/"),
            Token::Percent => Some("%"),
            _              => None,
        } {
            let start = expr.span(); self.advance();
            let right = self.parse_unary()?;
            let span  = start.merge(right.span());
            expr = Expr::BinaryOp { left: Box::new(expr), op: op.into(), right: Box::new(right), span };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        let start = self.current_span();
        match self.peek_token() {
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span());
                Ok(Expr::UnaryOp { op: "!".into(), operand: Box::new(operand), span })
            }
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span());
                Ok(Expr::UnaryOp { op: "-".into(), operand: Box::new(operand), span })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_token() {
                Token::LParen => {
                    self.advance();
                    let arguments = self.parse_arg_list()?;
                    self.consume(Token::RParen)?;
                    let span = expr.span().merge(self.prev_span());
                    expr = Expr::Call { function: Box::new(expr), arguments, span };
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.consume(Token::RBracket)?;
                    let span = expr.span().merge(self.prev_span());
                    expr = Expr::Index { object: Box::new(expr), index: Box::new(index), span };
                }
                Token::Dot => {
                    self.advance();
                    let member = self.consume_ident()?;
                    if matches!(self.peek_token(), Token::LParen) {
                        self.advance();
                        let arguments = self.parse_arg_list()?;
                        self.consume(Token::RParen)?;
                        let span = expr.span().merge(self.prev_span());
                        expr = Expr::MethodCall { object: Box::new(expr), method: member, arguments, span };
                    } else {
                        let span = expr.span().merge(self.prev_span());
                        expr = Expr::MemberAccess { object: Box::new(expr), member, span };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();
        if matches!(self.peek_token(), Token::RParen) { return Ok(args); }
        loop {
            args.push(self.parse_expr()?);
            if !matches!(self.peek_token(), Token::Comma) { break; }
            self.advance();
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let start = self.current_span();
        match self.peek_token().clone() {
            Token::Number(n) => { self.advance(); Ok(Expr::Number { value: n, span: start }) }
            Token::String(s) => { self.advance(); Ok(Expr::String { value: s, span: start }) }
            Token::InterpolatedString(segments) => {
                self.advance();
                let mut parts = Vec::new();
                for (is_expr, text) in segments {
                    if is_expr {
                        let mut sub = Parser::new(&text, self.source_id);
                        let sub_items = sub.parse().map_err(|_| ParserError::InvalidSyntax {
                            context: format!("interpolated expression: {}", text),
                            span: start,
                        })?;
                        if let Some(Item::Statement(Stmt::Expr(e))) = sub_items.into_iter().next() {
                            parts.push(StringPart::Expr(e));
                        } else {
                            parts.push(StringPart::Literal(text));
                        }
                    } else {
                        parts.push(StringPart::Literal(text));
                    }
                }
                Ok(Expr::InterpolatedString { parts, span: start })
            }
            Token::True   => { self.advance(); Ok(Expr::Boolean { value: true,  span: start }) }
            Token::False  => { self.advance(); Ok(Expr::Boolean { value: false, span: start }) }
            Token::SelfKw => { self.advance(); Ok(Expr::SelfRef { span: start }) }
            Token::New    => {
                self.advance();
                let class_name = self.consume_ident()?;
                self.consume(Token::LParen)?;
                let arguments = self.parse_arg_list()?;
                self.consume(Token::RParen)?;
                Ok(Expr::New { class_name, arguments, span: start.merge(self.prev_span()) })
            }
            Token::Ident(id) => { self.advance(); Ok(Expr::Identifier { name: id, span: start }) }
            Token::In  => { self.advance(); Ok(Expr::Identifier { name: "in".into(),  span: start }) }
            Token::Out => { self.advance(); Ok(Expr::Identifier { name: "out".into(), span: start }) }
            Token::Dot => {
                // .member shorthand → self.member
                self.advance();
                let member = self.consume_ident()?;
                Ok(Expr::MemberAccess {
                    object: Box::new(Expr::SelfRef { span: start }),
                    member,
                    span: start.merge(self.prev_span()),
                })
            }
            Token::LBracket => {
                self.advance();
                let items = self.parse_list_items()?;
                self.consume(Token::RBracket)?;
                Ok(Expr::List { items, span: start.merge(self.prev_span()) })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParserError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{:?}", self.peek_token()),
                span: start,
            }),
        }
    }

    fn parse_list_items(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut items = Vec::new();
        if matches!(self.peek_token(), Token::RBracket) { return Ok(items); }
        loop {
            items.push(self.parse_expr()?);
            if !matches!(self.peek_token(), Token::Comma) { break; }
            self.advance();
        }
        Ok(items)
    }

    // -----------------------------------------------------------------------
    // Utility
    // -----------------------------------------------------------------------

    fn peek_token(&self) -> Token {
        self.tokens.front().map(|(t, _)| t.clone()).unwrap_or(Token::Eof)
    }
    fn prev_span(&self) -> Span {
        self.tokens.front().map(|(_, s)| *s)
            .unwrap_or_else(|| Span::new(self.source_id, 0, 0))
    }
    fn current_span(&self) -> Span {
        self.tokens.front().map(|(_, s)| *s)
            .unwrap_or_else(|| Span::new(self.source_id, 0, 0))
    }
    fn advance(&mut self) -> Token {
        self.tokens.pop_front().map(|(t, _)| t).unwrap_or(Token::Eof)
    }
    fn consume(&mut self, expected: Token) -> Result<(), ParserError> {
        let token = self.peek_token();
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
                span: self.current_span(),
            })
        }
    }
    fn consume_ident(&mut self) -> Result<String, ParserError> {
        match self.peek_token() {
            Token::Ident(id) => { self.advance(); Ok(id) }
            _ => Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", self.peek_token()),
                span: self.current_span(),
            }),
        }
    }
    fn is_at_end(&self) -> bool { matches!(self.peek_token(), Token::Eof) }
    fn peek_nth(&self, n: usize) -> Token {
        self.tokens.get(n).map(|(t, _)| t.clone()).unwrap_or(Token::Eof)
    }
    fn skip_semicolons(&mut self) {
        while matches!(self.peek_token(), Token::Semicolon) { self.advance(); }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Vec<Item> {
        Parser::new(src, 0).parse().expect("parse failed")
    }

    // ── out statement: space-separated args ─────────────────────────────────

    #[test]
    fn test_out_space_separated() {
        // `out "text" var "text"` should produce 3 arguments
        let src = r#"
            fun demo(n) {
                out "Calculating fib(" n ")...";
                out "Result: " n;
            }
        "#;
        let items = parse(src);
        assert_eq!(items.len(), 1);
        if let Item::FunctionDecl { body, .. } = &items[0] {
            // First out: 3 args
            if let Stmt::Out { arguments, .. } = &body[0] {
                assert_eq!(arguments.len(), 3, "Expected 3 args, got {}", arguments.len());
            } else { panic!("Expected Out stmt"); }
            // Second out: 2 args
            if let Stmt::Out { arguments, .. } = &body[1] {
                assert_eq!(arguments.len(), 2, "Expected 2 args, got {}", arguments.len());
            } else { panic!("Expected Out stmt"); }
        }
    }

    #[test]
    fn test_out_call_in_space_separated() {
        // `out "fib(" i ") = " fib(i)` → 4 args (last is a call)
        let src = r#"
            fun show(i) {
                out "fib(" i ") = " fib(i);
            }
        "#;
        let items = parse(src);
        if let Item::FunctionDecl { body, .. } = &items[0] {
            if let Stmt::Out { arguments, .. } = &body[0] {
                assert_eq!(arguments.len(), 4, "Expected 4 args, got {}", arguments.len());
            } else { panic!("Expected Out stmt"); }
        }
    }

    #[test]
    fn test_out_single_arg_unchanged() {
        let src = r#"fun f() { out "hello"; }"#;
        let items = parse(src);
        if let Item::FunctionDecl { body, .. } = &items[0] {
            if let Stmt::Out { arguments, .. } = &body[0] {
                assert_eq!(arguments.len(), 1);
            }
        }
    }

    #[test]
    fn test_out_comma_separated_unchanged() {
        // Comma-separated still works
        let src = r#"fun f(a, b) { out a, b; }"#;
        let items = parse(src);
        if let Item::FunctionDecl { body, .. } = &items[0] {
            if let Stmt::Out { arguments, .. } = &body[0] {
                assert_eq!(arguments.len(), 2);
            }
        }
    }

    // ── full fibonacci smoke test ────────────────────────────────────────────

    #[test]
    fn test_fibonacci_file() {
        let src = r#"
            fun fib(n) {
                if n == 0 {
                    ret 0;
                }
                if n == 1 {
                    ret 1;
                }
                ret fib(n - 1) + fib(n - 2);
            }

            let n = 10;
            let result = fib(n);

            out "=== Axiom Fibonacci Demo ===";
            out "Calculating fib(" n ")...";
            out "Result: " result;

            out "";
            out "Sequence:";

            fun print_seq(i, limit) {
                if i == limit {
                    ret;
                }
                out "fib(" i ") = " fib(i);
                print_seq(i + 1, limit);
            }

            print_seq(0, n + 1);

            out "";
            out "Demo Complete.";
        "#;
        // Must parse without error
        let items = Parser::new(src, 0).parse().expect("fibonacci must parse");

        // Declarations hoisted before statements
        let first_stmt = items.iter().position(|i| matches!(i, Item::Statement(_)));
        let last_decl  = items.iter().rposition(|i| !matches!(i, Item::Statement(_)));
        if let (Some(fs), Some(ld)) = (first_stmt, last_decl) {
            assert!(ld < fs, "decls must precede stmts");
        }
    }

    // ── out does not eat the next statement keyword ──────────────────────────

    #[test]
    fn test_out_stops_at_statement_keyword() {
        // `out "done"` followed by `out "next"` on next line — must be 2 stmts
        let src = r#"
            fun f() {
                out "done";
                out "next";
            }
        "#;
        let items = parse(src);
        if let Item::FunctionDecl { body, .. } = &items[0] {
            assert_eq!(body.len(), 2, "Should be 2 out statements");
        }
    }

    // ── hoisting ────────────────────────────────────────────────────────────

    #[test]
    fn test_hoisting_stmt_after_decls() {
        let src = r#"
            main();
            User { let name; init(n){ .name = n; } display(){ out .name; } }
            Status { Active, Inactive, }
            main() { out "hello"; }
        "#;
        let items = parse(src);
        let first_stmt = items.iter().position(|i| matches!(i, Item::Statement(_)));
        let last_decl  = items.iter().rposition(|i| !matches!(i, Item::Statement(_)));
        if let (Some(fs), Some(ld)) = (first_stmt, last_decl) {
            assert!(ld < fs, "declarations must precede statements");
        }
    }

    // ── basic items ─────────────────────────────────────────────────────────

    #[test] fn test_number()   { assert_eq!(parse("42").len(), 1); }
    #[test] fn test_function() { assert_eq!(parse("fun add(x,y){x+y}").len(), 1); }

    #[test] fn test_class() {
        let src = r#"cls A { fun init(self,n){self.name=n;} fun speak(self){out self.name;} }"#;
        assert!(matches!(parse(src)[0], Item::ClassDecl{..}));
    }

    #[test] fn test_class_ext() {
        let src = r#"cls Dog ext Animal { fun speak(self){out "Woof!";} }"#;
        if let Item::ClassDecl{parent,..} = &parse(src)[0] {
            assert_eq!(parent.as_deref(), Some("Animal"));
        }
    }

    #[test] fn test_enum() {
        let src = r#"enm Color { Red, Green, Blue }"#;
        if let Item::EnumDecl{variants,..} = &parse(src)[0] {
            assert_eq!(variants.len(), 3);
        }
    }

    // ── match patterns ──────────────────────────────────────────────────────

    #[test] fn test_if_as_match_bare() {
        parse(r#"fun f(s){if s{Active=>out "a",Inactive=>out "i",els=>out "?",};}"#);
    }

    #[test] fn test_if_as_match_dotted() {
        parse(r#"
            fun handle(status) {
                if status {
                    Status.Active   => out "Status: Active",
                    Status.Inactive => out "Status: Inactive",
                    Status.Pending  => out "Status: Pending",
                    els             => out "Unknown status",
                };
            }
        "#);
    }

    #[test] fn test_if_as_match_literal() {
        parse(r#"fun f(){if true{42=>out "yes",els=>out "no",};}"#);
    }

    // ── genesis syntax ──────────────────────────────────────────────────────

    #[test] fn test_genesis_class() {
        let src = r#"User{let name;let age;init(n,a){.name=n;.age=a;}display(){out .name;}}"#;
        assert!(matches!(parse(src)[0], Item::ClassDecl{..}));
    }

    #[test] fn test_genesis_enum() {
        assert!(matches!(parse("Status{Active,Inactive,Pending,}")[0], Item::EnumDecl{..}));
    }

    #[test] fn test_genesis_function() {
        assert!(matches!(parse("handle(s){out s;}")[0], Item::FunctionDecl{..}));
    }

    // ── bigdemo smoke test ───────────────────────────────────────────────────

    #[test] fn test_bigdemo_full() {
        let src = r#"
            main() {
                let x = 42;
                let greeting = "Axiom";
                out greeting.upper();
                out greeting.align(20, "right");
                let obj = User("Alice", 30);
                obj.display();
                let numbers = [1, 2, 3, 4, 5];
                out avg(numbers);
                out sqrt(16);
                let car = Car("Tesla", 2024);
                car.description();
                let status = Status.Active;
                handle_status(status);
                let items = [5, 2, 8, 1, 9];
                out items.len();
                items.push(10);
                out items;
                if true {
                    42  => out "The answer to everything!",
                    els => out "Something else",
                };
            }
            User { let name; let age; init(name,age){.name=name;.age=age;} display(){out .name;out .age;} }
            Car  { let make; let year; init(make,year){.make=make;.year=year;} description(){out .make;out .year;} }
            Status { Active, Inactive, Pending, }
            handle_status(status) {
                if status {
                    Status.Active   => out "Status: Active",
                    Status.Inactive => out "Status: Inactive",
                    Status.Pending  => out "Status: Pending",
                    els             => out "Unknown status",
                };
            }
            main();
        "#;
        let items = Parser::new(src, 0).parse().expect("bigdemo must parse");
        let first_stmt = items.iter().position(|i| matches!(i, Item::Statement(_))).unwrap();
        let last_decl  = items.iter().rposition(|i| !matches!(i, Item::Statement(_))).unwrap();
        assert!(last_decl < first_stmt);
    }
}
