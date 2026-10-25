#pragma once
#include <string>
#include <vector>

struct Token {
    std::string kind;
    double value = 0;
    long long ival = 0;
    std::vector<int> strvals;
    std::string name;
    int line=0,col=0;
};

class Lexer {
public:
    explicit Lexer(const std::string &s);
    std::vector<Token> tokenize();
private:
    const std::string &source;
    size_t i=0; int line=1,col=1;
    char peek(int o=0) const; char advance(); bool eof() const;
};
