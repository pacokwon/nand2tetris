use crate::ast::{Expr, ExprTerm, Statement, SubroutineCall};
use crate::lexer::Lexer;
use crate::token::{KeywordType, TokenType};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        lexer.advance_token();
        lexer.advance_token();
        Parser { lexer }
    }

    pub fn parse(&mut self) {
        // let mut parsed = Vec::new();
        self.parse_class()
    }

    fn advance(&mut self) -> TokenType {
        self.lexer.advance_token().token_type
    }

    fn parse_class(&mut self) {
        todo!()
    }
    fn parse_class_var_dec(&mut self) {
        todo!()
    }
    fn parse_subroutine_dec(&mut self) {
        todo!()
    }
    fn parse_parameter_list(&mut self) {
        todo!()
    }
    fn parse_subroutine_body(&mut self) {
        todo!()
    }

    fn parse_var_dec(&mut self) {
        todo!()
    }

    fn parse_statements(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();

        loop {
            let stmt = match self.lexer.get_current_token_type() {
                TokenType::Keyword(KeywordType::Let) => {
                    self.advance();
                    self.parse_let_statement()
                },
                TokenType::Keyword(KeywordType::If) => {
                    self.advance();
                    self.parse_if_statement()
                },
                TokenType::Keyword(KeywordType::While) => {
                    self.advance();
                    self.parse_while_statement()
                },
                TokenType::Keyword(KeywordType::Do) => {
                    self.advance();
                    self.parse_do_statement()
                },
                TokenType::Keyword(KeywordType::Return) => {
                    self.advance();
                    self.parse_return_statement()
                },
                _ => break,
            };
            statements.push(stmt);
        }

        statements
    }

    fn parse_while_statement(&mut self) -> Statement {
        let TokenType::Symbol('(') = self.lexer.get_current_token_type() else {
            panic!("Expected '(' after 'while'. Encountered {:?}", self.lexer.get_current_token_type())
        };
        self.advance();

        let condition = self.parse_expression();

        let TokenType::Symbol(')') = self.lexer.get_current_token_type() else {
            panic!("Expected ')' after while condition. Encountered {:?}", self.lexer.get_current_token_type())
        };
        self.advance();

        let TokenType::Symbol('{') = self.lexer.get_current_token_type() else {
            panic!("Expected '{{' before while body. Encountered {:?}", self.lexer.get_current_token_type())
        };
        self.advance();

        let statements = self.parse_statements();

        let TokenType::Symbol('}') = self.lexer.get_current_token_type() else {
            panic!("Expected '}}' after while body. Encountered {:?}", self.lexer.get_current_token_type())
        };
        self.advance();

        Statement::While {
            condition,
            statements,
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        let TokenType::Symbol('(') = self.advance() else {
            panic!("Expected '(' after 'if'.")
        };

        let condition = self.parse_expression();

        let tt = self.advance();
        let TokenType::Symbol(')') = tt else {
            panic!("Expected ')' after conditional. Encountered {:?}", tt);
        };
        let tt = self.advance();
        let TokenType::Symbol('{') = tt else {
            panic!("Expected '{{' before if conditional. Encountered {:?}", tt);
        };

        let if_true = self.parse_statements();

        let tt = self.advance();
        let TokenType::Symbol('}') = tt else {
            panic!("Expected '}}' after if_true body. Encountered {:?}", tt);
        };

        let if_false = if let TokenType::Keyword(KeywordType::Else) =
            self.lexer.get_current_token_type()
        {
            self.advance();
            let tt = self.advance();
            let TokenType::Symbol('{') = tt else {
                panic!("Expected '{{' after 'else'. Encountered {:?}", tt);
            };

            let if_false = self.parse_statements();

            let tt = self.advance();
            let TokenType::Symbol('}') = tt else {
                panic!("Expected '}}' after if_false body. Encountered {:?}", tt);
            };

            Some(if_false)
        } else {
            None
        };

        Statement::If {
            condition,
            if_true,
            if_false,
        }
    }

    fn parse_return_statement(&mut self) -> Statement {
        if let TokenType::Symbol(';') = self.lexer.get_current_token_type() {
            self.advance();
            return Statement::Return { value: None };
        }

        let expr = self.parse_expression();
        let TokenType::Symbol(';') = self.lexer.get_current_token_type() else { panic!("Expected ';' after return statement. Encountered {:?}", self.lexer.get_current_token_type()) };
        self.advance();
        Statement::Return { value: Some(expr) }
    }

    fn parse_let_statement(&mut self) -> Statement {
        let tt = self.advance();
        let TokenType::Identifier(v) = tt else {
            panic!("Expected identifier while parsing let statement. Encountered {:?}", tt)
        };
        let name = v.clone();

        let access = if let TokenType::Symbol('[') = self.lexer.get_current_token_type() {
            self.advance();
            let expr = self.parse_expression();

            let tt = self.advance();
            let TokenType::Symbol(']') = tt else {
                panic!("Expected ']' after index expression. Encountered {:?}", tt);
            };
            Some(expr)
        } else {
            None
        };

        let tt = self.advance();
        let TokenType::Symbol('=') = tt else {
            panic!("Expected '=' after variable name. Encountered {:?}", tt)
        };

        let expr = self.parse_expression();
        let tt = self.advance();
        let TokenType::Symbol(';') = tt else { panic!("Expected ';' after variable assignment. Encountered {:?}", tt) };

        Statement::Let { name, access, expr }
    }

    fn parse_do_statement(&mut self) -> Statement {
        let tt = self.advance();
        let TokenType::Identifier(ref n) = tt else {
            panic!("Expected subroutine name after 'do'. Encountered {:?}", tt);
        };
        let name = n.clone();

        let tt = self.advance();
        let call = match tt {
            // function call
            TokenType::Symbol('(') => {
                let expr_list = self.parse_expression_list();
                SubroutineCall::Function(name, expr_list)
            }
            // method call
            TokenType::Symbol('.') => {
                let TokenType::Identifier(ref m) = self.advance() else {
                    panic!("Expected identifier after '.'. Encountered {:?}", self.lexer.get_current_token_type());
                };
                let method_name = m.clone();

                let TokenType::Symbol('(') = self.advance() else {
                    panic!("Expected '(' after method name. Encountered {:?}", self.lexer.get_current_token_type());
                };

                let expr_list = self.parse_expression_list();
                SubroutineCall::Method(name, method_name, expr_list)
            }
            _ => panic!(
                "Expected subroutine call after 'do'. Encountered {:?}",
                self.lexer.get_current_token_type()
            ),
        };
        let tt = self.advance();
        let TokenType::Symbol(';') = tt else {
            panic!("Expected ;' after do statement.");
        };

        Statement::Do { call }
    }

    fn parse_expression(&mut self) -> Expr {
        let lhs = self.parse_term();

        let mut rhs = Vec::new();
        loop {
            use TokenType::*;
            match self.lexer.get_current_token_type() {
                Symbol('+') | Symbol('-') | Symbol('*') | Symbol('/') | Symbol('&')
                | Symbol('|') | Symbol('<') | Symbol('>') | Symbol('=') => {
                    let Symbol(c) = self.advance() else { unreachable!("Token type must be a symbol here.") };
                    let t = self.parse_term();
                    rhs.push((c, t));
                }
                _ => break,
            }
        }

        Expr { lhs, rhs }
    }

    fn parse_term(&mut self) -> ExprTerm {
        let tt = self.advance();
        match tt {
            TokenType::Integer(num) => {
                ExprTerm::Integer(num)
            }
            TokenType::String(ref s) => {
                ExprTerm::Str(s.clone())
            }
            TokenType::Keyword(typ) => {
                match typ {
                    KeywordType::True => ExprTerm::True,
                    KeywordType::False => ExprTerm::False,
                    KeywordType::Null => ExprTerm::Null,
                    KeywordType::This => ExprTerm::This,
                    t => panic!("Unexpected keyword literal {:?} encountered.", t),
                }
            }
            TokenType::Identifier(ref name) => {
                let var_name = name.clone();
                match self.lexer.get_current_token_type() {
                    // array access
                    TokenType::Symbol('[') => {
                        self.advance();
                        let expr = self.parse_expression();
                        ExprTerm::Access(var_name, Box::new(expr))
                    }
                    // function call
                    TokenType::Symbol('(') => {
                        self.advance();
                        let expr_list = self.parse_expression_list();
                        ExprTerm::Call(SubroutineCall::Function(var_name, expr_list))
                    }
                    // method call
                    TokenType::Symbol('.') => {
                        self.advance();
                        let TokenType::Identifier(ref m) = self.lexer.get_current_token_type() else {
                            panic!("Expected identifier after '.'. Encountered {:?}", self.lexer.get_current_token_type());
                        };
                        let method_name = m.clone();

                        self.advance();
                        let TokenType::Symbol('(') = self.lexer.get_current_token_type() else {
                            panic!("Expected '(' after method name. Encountered {:?}", self.lexer.get_current_token_type());
                        };

                        self.advance();
                        let expr_list = self.parse_expression_list();
                        ExprTerm::Call(SubroutineCall::Method(var_name, method_name, expr_list))
                    }
                    _ => ExprTerm::Variable(var_name),
                }
            }
            TokenType::Symbol('(') => {
                let expr = self.parse_expression();
                let tt = self.advance();
                let TokenType::Symbol(')') = tt else {
                    panic!("Expected ')' after parenthesized expression. Encountered {:?}", tt);
                };
                ExprTerm::Group(Box::new(expr))
            }
            TokenType::Symbol('-') => {
                let term = self.parse_term();
                ExprTerm::Unary('-', Box::new(term))
            }
            TokenType::Symbol('~') => {
                let term = self.parse_term();
                ExprTerm::Unary('~', Box::new(term))
            }
            ref t => panic!("Encountered invalid token type while parsing term: {:?}", t),
        }
    }

    fn parse_expression_list(&mut self) -> Vec<Expr> {
        if let TokenType::Symbol(')') = self.lexer.get_current_token_type() {
            self.advance();
            return Vec::new();
        }

        let expr = self.parse_expression();
        let mut list = vec![expr];

        while let TokenType::Symbol(',') = self.lexer.get_current_token_type() {
            self.advance();
            let expr = self.parse_expression();
            list.push(expr);
        }

        if let TokenType::Symbol(')') = self.lexer.get_current_token_type() {
            self.advance();
            list
        } else {
            panic!(
                "Expected ')' after expression list. Encounterd {:?}",
                self.lexer.current_token.token_type
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::ast;
    use crate::ast::ExprTerm;
    use crate::ast::Statement;
    use crate::ast::SubroutineCall;

    #[test]
    fn test_term_int() {
        use ast::ExprTerm::*;
        let input = "30";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();
        if let Integer(num) = term {
            assert_eq!(num, 30);
        } else {
            panic!("Wrong variant encountered.");
        }
    }

    #[test]
    fn test_term_str() {
        use ast::ExprTerm::*;
        let input = "\"foobar\"";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();
        if let Str(ref v) = term {
            assert_eq!(v, "foobar");
        } else {
            panic!("Wrong variant encountered.");
        }
    }

    #[test]
    fn test_term_keyword() {
        use ast::ExprTerm::*;
        let input = "true";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();
        let True = term else {
            panic!("Wrong variant encountered.");
        };
    }

    #[test]
    fn test_term_variable() {
        use ast::ExprTerm::*;

        let input = "foo";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();

        let Variable(s) = term else { panic!("Must be variable.") };
        assert_eq!(s, "foo");
    }

    #[test]
    fn test_term_access() {
        use ast::ExprTerm::*;

        let input = "foo[3]";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();

        let Access(v, b) = term else { panic!("Must be access.") };
        assert_eq!(v, "foo");
        let Integer(3) = b.lhs else { panic!("Must be 3.") };
    }

    #[test]
    fn test_term_unary() {
        use ast::ExprTerm::*;

        let input = "-spam";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();

        let Unary('-', t) = term else { panic!("Must be unary.") };
        let Variable(ref s) = *t else { panic!("Must be variable.") };
        assert_eq!(s, "spam");
    }

    #[test]
    fn test_term_function_empty() {
        use ast::ExprTerm::*;

        let input = "foo()";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();

        let Call(SubroutineCall::Function(s, v)) = term else { panic!("Must be call.") };

        assert_eq!(s, "foo");
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_term_function_call() {
        use ast::ExprTerm::*;

        let input = "foo(3)";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();
        let Call(SubroutineCall::Function(s, v)) = term else { panic!("Must be call.") };

        assert_eq!(s, "foo");
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_term_method_call() {
        use ast::ExprTerm::*;

        let input = "foo.bar(3)";
        let mut parser = Parser::new(input);
        let term = parser.parse_term();
        let Call(SubroutineCall::Method(s, v, args)) = term else { panic!("Must be call.") };

        assert_eq!(s, "foo");
        assert_eq!(v, "bar");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_expression_int() {
        use ast::ExprTerm::*;
        let input = "3";
        let mut parser = Parser::new(input);
        let term = parser.parse_expression();

        let Integer(3) = term.lhs else {
            panic!("Must be 3.");
        };
    }

    #[test]
    fn test_expression_binary1() {
        use ast::ExprTerm::*;
        let input = "3 + 4";
        let mut parser = Parser::new(input);
        let term = parser.parse_expression();
        let Integer(3) = term.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(term.rhs.len(), 1);

        assert_eq!(term.rhs[0].0, '+');
        let Integer(4) = term.rhs[0].1 else {
            panic!("Must be 4.");
        };
    }

    #[test]
    fn test_expression_binary2() {
        use ast::ExprTerm::*;
        let input = "foobar / 4";
        let mut parser = Parser::new(input);
        let term = parser.parse_expression();
        let Variable(s) = term.lhs else {
            panic!("Must be variable.");
        };
        assert_eq!(s, "foobar");

        assert_eq!(term.rhs[0].0, '/');
        let Integer(4) = term.rhs[0].1 else {
            panic!("Must be 4.");
        };
    }

    #[test]
    fn test_expression_group() {
        use ast::ExprTerm::*;
        let input = "3 * (4 + 5)";
        let mut parser = Parser::new(input);
        let term = parser.parse_expression();
        let Integer(3) = term.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(term.rhs.len(), 1);

        assert_eq!(term.rhs[0].0, '*');
        let Group(inner) = &term.rhs[0].1 else {
            panic!("Must be a group expression.");
        };

        let Integer(4) = inner.lhs else {
            panic!("Must be 4.");
        };
        assert_eq!(inner.rhs.len(), 1);

        assert_eq!(inner.rhs[0].0, '+');
        let Integer(5) = inner.rhs[0].1 else {
            panic!("Must be 5.");
        };
    }

    #[test]
    fn test_statement_let1() {
        let input = "let x = 3;";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();
        assert_eq!(stmts.len(), 1);
        let Statement::Let { name, access, expr } = &stmts[0] else {
            panic!("Must be let.");
        };

        assert_eq!(name, "x");
        let None = access else {
            panic!("Must be none.");
        };
        let ExprTerm::Integer(3) = expr.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(expr.rhs.len(), 0);
    }

    #[test]
    fn test_statement_let2() {
        let input = "let x[42] = 3;";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();
        assert_eq!(stmts.len(), 1);
        let Statement::Let { name, access, expr } = &stmts[0] else {
            panic!("Must be let.");
        };

        assert_eq!(name, "x");
        let Some(index_expr) = access else {
            panic!("Must be none.");
        };
        let ExprTerm::Integer(42) = index_expr.lhs else {
            panic!("Must be 42.");
        };
        assert_eq!(expr.rhs.len(), 0);

        let ExprTerm::Integer(3) = expr.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(expr.rhs.len(), 0);
    }

    #[test]
    fn test_statement_do() {
        let input = "do foobar(3 + 4, 2 * 5); do foo.bar();";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();

        assert_eq!(stmts.len(), 2);
        let Statement::Do { call } = &stmts[0] else {
            panic!("Must be do statement.");
        };
        let SubroutineCall::Function(n, args) = call else {
            panic!("Must be function call.");
        };
        assert_eq!(n, "foobar");
        assert_eq!(args.len(), 2);
        let ExprTerm::Integer(3) = args[0].lhs else {
            panic!("First argument must be 3.");
        };

        let Statement::Do { call } = &stmts[1] else {
            panic!("Must be do statement.");
        };
        let SubroutineCall::Method(first, second, args) = call else {
            panic!("Must be method call.");
        };
        assert_eq!(first, "foo");
        assert_eq!(second, "bar");
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn test_statement_if() {
        let input = "if (x){let y = 0;} if(3){let x = 0;} else  {let z = 3;}";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();

        assert_eq!(stmts.len(), 2);

        let Statement::If { condition, if_true, if_false } = &stmts[0] else {
            panic!("Must be if statement.");
        };
        let ExprTerm::Variable(ref s) = condition.lhs else {
            panic!("Must be variable.");
        };
        assert_eq!(s, "x");
        assert_eq!(if_true.len(), 1);
        let None = if_false else {
            panic!("There is no false branch for this statement.");
        };

        let Statement::If { condition, if_true, if_false } = &stmts[1] else {
            panic!("Must be if statement.");
        };
        let ExprTerm::Integer(3) = condition.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(if_true.len(), 1);
        assert_eq!(if_false.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_statement_while() {
        let input = "while (3 = 4) {
            do foo.bar(x, y, z);
            let spam = 3;
        }";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();

        assert_eq!(stmts.len(), 1);
        let Statement::While { condition, statements: body } = &stmts[0] else {
            panic!("Must be while statement.");
        };

        let ExprTerm::Integer(3) = condition.lhs else {
            panic!("Must be 3.");
        };
        assert_eq!(condition.rhs.len(), 1);
        assert_eq!(condition.rhs[0].0, '=');
        let ExprTerm::Integer(4) = condition.rhs[0].1 else {
            panic!("Must be 4.");
        };

        assert_eq!(body.len(), 2);
        let Statement::Do {..} = body[0] else {
            panic!("Must be do statment.");
        };
        let Statement::Let {..} = body[1] else {
            panic!("Must be let statment.");
        };
    }

    #[test]
    fn test_statement_return() {
        let input = "return this; return;";
        let mut parser = Parser::new(input);
        let stmts = parser.parse_statements();

        assert_eq!(stmts.len(), 2);
        let Statement::Return { value } = &stmts[0] else {
            panic!("Must be return statement.");
        };

        let Some(val) = value else {
            panic!("There must be a return value");
        };
        assert_eq!(val.rhs.len(), 0);
        let ExprTerm::This = val.lhs else {
            panic!("Must be `this`.");
        };

        let Statement::Return { value } = &stmts[1] else {
            panic!("Must be return statement.");
        };
        let None = value else {
            panic!("Must be None.");
        };
    }
}
