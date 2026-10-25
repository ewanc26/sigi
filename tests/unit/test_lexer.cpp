#include <catch2/catch_test_macros.hpp>
#include "../../src/lexer/lexer.h"

TEST_CASE("lexer reads a numeric literal") {
    Lexer lexer("!42");
    auto tokens = lexer.tokenize();
    REQUIRE(tokens.size() == 1);
    REQUIRE(tokens[0].kind == "NUM");
}

TEST_CASE("lexer rejects unknown characters instead of dropping them") {
    Lexer lexer("q");
    REQUIRE_THROWS(lexer.tokenize());
}
