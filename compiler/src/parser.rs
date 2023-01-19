use crate::ast::{
    Class, ClassVarDec, Expr, ExprTerm, Statement, SubroutineBody, SubroutineCall, SubroutineDec,
    SubroutineKind, VarDec, VariableScope, VariableType,
};
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

    pub fn parse(&mut self) -> Class {
        self.parse_class()
    }

    fn advance(&mut self) -> TokenType {
        self.lexer.advance_token().token_type
    }

    fn parse_class(&mut self) -> Class {
        let tt = self.advance();
        let TokenType::Keyword(KeywordType::Class) = tt else {
            panic!("Expected 'class' while parsing class. Encountered {:?}", tt);
        };

        let name = self.parse_identifier();

        self.consume_symbol('{');

        let variables = self.parse_class_var_decs();
        let subroutines = self.parse_subroutine_decs();

        self.consume_symbol('}');

        Class {
            name,
            variables,
            subroutines,
        }
    }

    fn parse_class_var_decs(&mut self) -> Vec<ClassVarDec> {
        let mut class_var_decs = Vec::new();

        loop {
            let tt = self.lexer.get_current_token_type();
            let scope = match tt {
                TokenType::Keyword(KeywordType::Static) => VariableScope::Static,
                TokenType::Keyword(KeywordType::Field) => VariableScope::Field,
                _ => break,
            };
            self.advance();
            let typ = self.parse_type();

            let var = self.parse_identifier();
            let mut vars = vec![var];

            while let TokenType::Symbol(',') = self.lexer.get_current_token_type() {
                self.advance();

                let var = self.parse_identifier();
                vars.push(var);
            }

            self.consume_symbol(';');

            let dec = ClassVarDec { scope, typ, vars };
            class_var_decs.push(dec);
        }

        class_var_decs
    }

    fn parse_subroutine_decs(&mut self) -> Vec<SubroutineDec> {
        let mut subroutines = Vec::new();

        loop {
            let tt = self.lexer.get_current_token_type();
            let kind = match tt {
                TokenType::Keyword(KeywordType::Constructor) => SubroutineKind::Constructor,
                TokenType::Keyword(KeywordType::Function) => SubroutineKind::Function,
                TokenType::Keyword(KeywordType::Method) => SubroutineKind::Method,
                _ => break,
            };
            self.advance();

            let tt = self.advance();
            let return_type = match tt {
                TokenType::Keyword(KeywordType::IntType) => VariableType::Int,
                TokenType::Keyword(KeywordType::BoolType) => VariableType::Boolean,
                TokenType::Keyword(KeywordType::CharType) => VariableType::Char,
                TokenType::Keyword(KeywordType::Void) => VariableType::Void,
                TokenType::Identifier(ref n) => VariableType::Other(n.clone()),
                _ => panic!("Expected return type. Encountered {:?}", tt),
            };

            let name = self.parse_identifier();

            self.consume_symbol('(');
            let parameters = self.parse_parameter_list();

            let body = self.parse_subroutine_body();

            let subroutine = SubroutineDec {
                kind,
                return_type,
                name,
                parameters,
                body,
            };
            subroutines.push(subroutine);
        }

        subroutines
    }

    fn parse_parameter_list(&mut self) -> Vec<(VariableType, String)> {
        if let TokenType::Symbol(')') = self.lexer.get_current_token_type() {
            self.advance();
            return Vec::new();
        }

        let typ = self.parse_type();
        let name = self.parse_identifier();

        let mut parameters = vec![(typ, name)];

        while let TokenType::Symbol(',') = self.lexer.get_current_token_type() {
            self.advance();
            let typ = self.parse_type();
            let name = self.parse_identifier();
            parameters.push((typ, name));
        }

        self.consume_symbol(')');
        parameters
    }

    fn parse_subroutine_body(&mut self) -> SubroutineBody {
        self.consume_symbol('{');

        let locals = self.parse_var_decs();
        let statements = self.parse_statements();

        self.consume_symbol('}');

        SubroutineBody { locals, statements }
    }

    fn parse_var_decs(&mut self) -> Vec<VarDec> {
        let mut decs = vec![];
        loop {
            let TokenType::Keyword(KeywordType::Var) = self.lexer.get_current_token_type() else {
                break
            };
            self.advance();

            let typ = self.parse_type();

            let var = self.parse_identifier();
            let mut vars = vec![var];
            while let TokenType::Symbol(',') = self.lexer.get_current_token_type() {
                self.advance();
                let var = self.parse_identifier();
                vars.push(var);
            }
            self.consume_symbol(';');

            let var_dec = VarDec { typ, vars };
            decs.push(var_dec);
        }
        decs
    }

    fn parse_type(&mut self) -> VariableType {
        let tt = self.advance();
        match tt {
            TokenType::Keyword(KeywordType::IntType) => VariableType::Int,
            TokenType::Keyword(KeywordType::BoolType) => VariableType::Boolean,
            TokenType::Keyword(KeywordType::CharType) => VariableType::Char,
            TokenType::Identifier(ref n) => VariableType::Other(n.clone()),
            _ => panic!("Expected type. Encountered {:?}", tt),
        }
    }

    fn parse_identifier(&mut self) -> String {
        let tt = self.advance();
        match tt {
            TokenType::Identifier(ref n) => n.clone(),
            _ => panic!("Expected identifier. Encountered {:?}", tt),
        }
    }

    fn consume_symbol(&mut self, expected: char) {
        let tt = self.advance();
        match tt {
            TokenType::Symbol(c) => {
                if c != expected {
                    panic!("Expected '{expected}'. Encountered {c}")
                }
            }
            _ => panic!("Expected '{expected}'. Encountered {:?}", tt),
        }
    }

    fn parse_statements(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();

        loop {
            let stmt = match self.lexer.get_current_token_type() {
                TokenType::Keyword(KeywordType::Let) => {
                    self.advance();
                    self.parse_let_statement()
                }
                TokenType::Keyword(KeywordType::If) => {
                    self.advance();
                    self.parse_if_statement()
                }
                TokenType::Keyword(KeywordType::While) => {
                    self.advance();
                    self.parse_while_statement()
                }
                TokenType::Keyword(KeywordType::Do) => {
                    self.advance();
                    self.parse_do_statement()
                }
                TokenType::Keyword(KeywordType::Return) => {
                    self.advance();
                    self.parse_return_statement()
                }
                _ => break,
            };
            statements.push(stmt);
        }

        statements
    }

    fn parse_while_statement(&mut self) -> Statement {
        self.consume_symbol('(');

        let condition = self.parse_expression();

        self.consume_symbol(')');
        self.consume_symbol('{');

        let statements = self.parse_statements();

        self.consume_symbol('}');

        Statement::While {
            condition,
            statements,
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.consume_symbol('(');

        let condition = self.parse_expression();

        self.consume_symbol(')');
        self.consume_symbol('{');

        let if_true = self.parse_statements();

        self.consume_symbol('}');

        let if_false =
            if let TokenType::Keyword(KeywordType::Else) = self.lexer.get_current_token_type() {
                self.advance();
                self.consume_symbol('{');

                let if_false = self.parse_statements();

                self.consume_symbol('}');

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
        self.consume_symbol(';');
        Statement::Return { value: Some(expr) }
    }

    fn parse_let_statement(&mut self) -> Statement {
        let name = self.parse_identifier();

        let access = if let TokenType::Symbol('[') = self.lexer.get_current_token_type() {
            self.advance();
            let expr = self.parse_expression();

            self.consume_symbol(']');
            Some(expr)
        } else {
            None
        };

        self.consume_symbol('=');
        let expr = self.parse_expression();
        self.consume_symbol(';');

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
                let method_name = self.parse_identifier();

                self.consume_symbol('(');
                let expr_list = self.parse_expression_list();
                SubroutineCall::Method(name, method_name, expr_list)
            }
            _ => panic!(
                "Expected subroutine call after 'do'. Encountered {:?}",
                self.lexer.get_current_token_type()
            ),
        };
        self.consume_symbol(';');

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
            TokenType::Integer(num) => ExprTerm::Integer(num),
            TokenType::String(ref s) => ExprTerm::Str(s.clone()),
            TokenType::Keyword(typ) => match typ {
                KeywordType::True => ExprTerm::True,
                KeywordType::False => ExprTerm::False,
                KeywordType::Null => ExprTerm::Null,
                KeywordType::This => ExprTerm::This,
                t => panic!("Unexpected keyword literal {:?} encountered.", t),
            },
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
                self.consume_symbol(')');
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

        self.consume_symbol(')');
        list
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::ast::ExprTerm;
    use crate::ast::Statement;
    use crate::ast::SubroutineCall;
    use crate::ast::SubroutineKind;
    use crate::ast::VariableScope;
    use crate::ast::VariableType;
    use crate::parser::Parser;

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

    #[test]
    fn test_class_var_decs() {
        let input = "
            field int data, foo;
            field List next;
            static char foobar;
        ";
        let mut parser = Parser::new(input);
        let decs = parser.parse_class_var_decs();

        assert_eq!(decs.len(), 3);

        assert_eq!(decs[0].scope, VariableScope::Field);
        assert_eq!(decs[0].typ, VariableType::Int);
        assert_eq!(decs[0].vars.len(), 2);

        assert_eq!(decs[1].scope, VariableScope::Field);
        assert_eq!(decs[1].typ, VariableType::Other(String::from("List")));
        assert_eq!(decs[1].vars.len(), 1);

        assert_eq!(decs[2].scope, VariableScope::Static);
        assert_eq!(decs[2].typ, VariableType::Char);
        assert_eq!(decs[2].vars.len(), 1);
    }

    #[test]
    fn test_subroutine_decs() {
        let input = "
            constructor List new(int car, List cdr) {
                let data = car;
                let next = cdr;
                return this;
            }

            method int getData() {
                var int x, y, z;
            }
            method int getNext() { return next; }

            function int func() { return 3; }
        ";
        let mut parser = Parser::new(input);
        let decs = parser.parse_subroutine_decs();

        assert_eq!(decs.len(), 4);

        assert_eq!(decs[0].kind, SubroutineKind::Constructor);
        assert_eq!(decs[0].return_type, VariableType::Other(String::from("List")));
        assert_eq!(decs[0].name, "new");
        assert_eq!(decs[0].parameters.len(), 2);
        assert_eq!(decs[0].body.locals.len(), 0);
        assert_eq!(decs[0].body.statements.len(), 3);

        assert_eq!(decs[1].kind, SubroutineKind::Method);
        assert_eq!(decs[1].return_type, VariableType::Int);
        assert_eq!(decs[1].name, "getData");
        assert_eq!(decs[1].parameters.len(), 0);
        assert_eq!(decs[1].body.locals.len(), 1);
        assert_eq!(decs[1].body.locals[0].vars.len(), 3);
        assert_eq!(decs[1].body.statements.len(), 0);

        assert_eq!(decs[2].kind, SubroutineKind::Method);
        assert_eq!(decs[2].return_type, VariableType::Int);
        assert_eq!(decs[2].name, "getNext");
        assert_eq!(decs[2].parameters.len(), 0);
        assert_eq!(decs[2].body.locals.len(), 0);
        assert_eq!(decs[2].body.statements.len(), 1);

        assert_eq!(decs[3].kind, SubroutineKind::Function);
        assert_eq!(decs[3].return_type, VariableType::Int);
        assert_eq!(decs[3].name, "func");
        assert_eq!(decs[3].parameters.len(), 0);
        assert_eq!(decs[3].body.locals.len(), 0);
        assert_eq!(decs[3].body.statements.len(), 1);
    }

    #[test]
    fn test_class() {
        let input = "
            class List {
                field int data;          // a list consists of a data field,
                field List next;         // followed by a list

                /* Creates a List. */
                constructor List new(int car, List cdr) {
                    let data = car;       // the identifiers car and cdr are used in
                    let next = cdr;       // memory of the Lisp programming language
                    return this;
                }

                /** Accessors. */
                method int getData() { return data; }
                method int getNext() { return next; }

                /** Prints this list. */
                method void print() {
                    var List current;    // initializes current to the first item
                    let current = this;  // of this list
                    while (~(current = null)) {
                        do Output.printInt(current.getData());
                        do Output.printChar(32); // prints a space
                        let current = current.getNext();
                    }
                    return;
                }

                /** Disposes this List by recursively disposing its tail. */
                method void dispose() {
                    if (~(next = null)) {
                        do next.dispose();
                    }
                    // Uses an OS routine to recycle this object.
                    do Memory.deAlloc(this);
                    return;
                }

                // More list processing methods can come here.

            }
        ";
        let mut parser = Parser::new(input);
        let class = parser.parse_class();

        assert_eq!(class.name, "List");
        assert_eq!(class.variables.len(), 2);
        assert_eq!(class.variables[0].scope, VariableScope::Field);
        assert_eq!(class.variables[0].typ, VariableType::Int);
        assert_eq!(class.variables[0].vars.len(), 1);
        assert_eq!(class.variables[0].vars[0], "data");
        assert_eq!(class.subroutines.len(), 5);
    }
}
