#include <catch2/catch_test_macros.hpp>
#include "../../src/codegen/codegen.h"

TEST_CASE("code generator rejects operations without a lowering implementation") {
    Program program;
    SOp unsupported;
    unsupported.kind = SOp::Continue;
    program.mainOps.push_back(unsupported);
    REQUIRE_THROWS(emitSigi(program));
}
