#include <catch2/catch_test_macros.hpp>
#include "../../src/lexer/lexer.h"
TEST_CASE("lexer num") { Lexer l("!42"); auto t=l.tokenize(); REQUIRE(t[0].kind=="NUM"); }
