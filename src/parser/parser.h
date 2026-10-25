#pragma once
#include "../lexer/lexer.h"
#include "../ast/ast.h"
class Parser {
public:
    explicit Parser(const std::vector<Token>&);
    Program parse();
private:
    const std::vector<Token> &tokens;
    size_t i=0;
    const Token& peek(int o=0) const;
    const Token& advance();
    bool eof() const;
};
