import subprocess
import sys
from pathlib import Path

import pytest

EXAMPLES_DIR = Path(__file__).parent.parent / "examples"


@pytest.mark.parametrize(
    "example_file",
    [
        "arithmetic.si",
        "control_flow.si",
        "factorial.si",
        "functions.si",
        "hello_world.si",
        "stack_ops.si",
        "while.si",
        "named_features.si",
    ],
)
def test_example(example_file):
    path = EXAMPLES_DIR / example_file
    # Use python -m sigi.main to run without needing to install the package
    cmd = [sys.executable, "-m", "sigi.main", str(path), "--run"]
    result = subprocess.run(cmd, capture_output=True, text=True)

    assert result.returncode == 0, f"Failed to run {example_file}:\n{result.stderr}"

    # Check output based on example name
    output = result.stdout.strip()
    if example_file == "arithmetic.si":
        assert output.splitlines() == ["1", "5", "7", "20", "5", "2"]
    elif example_file == "hello_world.si":
        assert output == "Hello, World!"
    elif example_file == "factorial.si":
        assert output == "120"
    elif example_file == "control_flow.si":
        lines = output.splitlines()
        assert lines[0] == "condition was true"
        assert lines[1] == "condition was false"
        assert lines[2:] == ["1", "1", "1", "1", "0"]
    elif example_file == "stack_ops.si":
        assert output.splitlines() == ["10", "1", "2", "2", "1", "42"]
    elif example_file == "while.si":
        assert output.splitlines() == ["5", "4", "3", "2", "1"]
    elif example_file == "functions.si":
        assert output.splitlines() == ["Function 0 says: 42", "Function 1 says: 99"]
    elif example_file == "named_features.si":
        assert output.splitlines() == ["Hello from .greet!", "123", "60"]


@pytest.mark.parametrize(
    "example_file",
    [
        "arithmetic.si",
        "named_features.si",
    ],
)
def test_interpreter(example_file):
    path = EXAMPLES_DIR / example_file
    cmd = [sys.executable, "-m", "sigi.main", str(path), "--interpret"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    assert result.returncode == 0
    if example_file == "named_features.si":
        assert result.stdout.strip().splitlines() == ["Hello from .greet!", "123", "60"]
