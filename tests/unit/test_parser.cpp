#include <catch2/catch_test_macros.hpp>
#include "../../src/lexer/lexer.h"
#include "../../src/parser/parser.h"

TEST_CASE("parser handles a basic numeric output program") {
    Lexer lexer("!5|");
    auto tokens = lexer.tokenize();
    Parser parser(tokens);
    auto program = parser.parse();
    REQUIRE(program.mainOps.size() == 2);
}

TEST_CASE("tilde maps to logical NOT") {
    Lexer lexer("~");
    auto tokens = lexer.tokenize();
    Parser parser(tokens);
    auto program = parser.parse();
    REQUIRE(program.mainOps.size() == 1);
    REQUIRE(program.mainOps.front().kind == SOp::Not);
}

TEST_CASE("parser rejects token kinds it does not implement") {
    Lexer lexer(";");
    auto tokens = lexer.tokenize();
    Parser parser(tokens);
    REQUIRE_THROWS(parser.parse());
}
