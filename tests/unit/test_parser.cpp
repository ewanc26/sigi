#include <catch2/catch_test_macros.hpp>
#include "../../src/lexer/lexer.h"
#include "../../src/parser/parser.h"
TEST_CASE("parser") { Lexer l("!5|"); auto t=l.tokenize(); Parser p(t); auto prog=p.parse(); REQUIRE(prog.mainOps.size()>0); }
