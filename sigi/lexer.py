"""Sigi lexer - pure symbolic tokenization."""

from dataclasses import dataclass
from typing import List, Optional, Union


@dataclass
class Token:
    kind: str
    value: Optional[Union[float, int, str]] = None
    line: int = 1
    col: int = 1

    def __repr__(self):
        if self.value is not None:
            return f"Token({self.kind!r}, {self.value!r})"
        return f"Token({self.kind!r})"


class LexError(Exception):
    pass


class Lexer:
    """
    Sigi lexer - all syntax is symbolic.

    Symbol reference:
    !N      Push number N (int or float)
    @       DUP - duplicate top
    #       SWAP - swap top two
    $       DROP - discard top
    +       ADD
    -       SUB
    *       MUL
    /       DIV
    %       MOD
    =       EQ - push 1 if equal, else 0
    <       LT - push 1 if a < b
    >       GT - push 1 if a > b
    ~       NOT - push 1 if 0, else 0
    |       Print number with newline
    ^       Print as character
    ?       Read number from input
    :       Store (pop val, pop addr, store to var)
    .       Load (pop addr, push var value)
    <n>     Push variable n directly (0-99)
    [ ]     WHILE loop - pop condition, loop while nonzero
    (N)     Call function N (0-99)
    { }     Block - for function definitions and conditionals
    ;       ELSE separator in blocks
    "text"  String literal
    'x      Character literal (no closing ')
    \\      End of line comment
    S       SIN - pop a, push sin(a)
    C       COS - pop a, push cos(a)
    T       TAN - pop a, push tan(a)
    R       SQRT - pop a, push sqrt(a)
    P       POW - pop exp, pop base, push pow(base, exp)
    F       FLOOR - pop a, push floor(a)
    L       LOG - pop a, push log(a)
    E       EXP - pop a, push exp(a)
    M       ABS - pop a, push fabs(a)
    N       ATAN2 - pop y, pop x, push atan2(y, x)
    W       RAND - push random value 0.0-1.0
    X       EXIT - pop code, exit program
    Z       TIME - push current time in seconds
    &       ALEN - pop arr_id, push array length
    A       ALOAD - pop idx, pop arr_id → push array[idx]
    a       ASTORE - pop val, pop idx, pop arr_id → array[idx] = val
    _       AINIT - pop arr_id, pop size → initialize array
    U       USLEEP - pop microseconds, sleep

    Tokens:
    NUM     - numeric value (from ! prefix)
    VAR     - variable index (digits directly)
    CHAR    - character literal
    STRING  - string literal
    BLOCK   - { start block
    ENDB    - } end block
    ELSE    - ; separator
    WHILE   - [ start loop
    WEND    - ] end loop
    CALL    - ( function call prefix
    All other symbols are single-char tokens
    """

    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.col = 1

    def _current(self) -> Optional[str]:
        return self.source[self.pos] if self.pos < len(self.source) else None

    def _peek(self, offset: int = 1) -> Optional[str]:
        idx = self.pos + offset
        return self.source[idx] if idx < len(self.source) else None

    def _advance(self) -> Optional[str]:
        ch = self._current()
        if ch == "\n":
            self.line += 1
            self.col = 1
        elif ch:
            self.col += 1
        self.pos += 1
        return ch

    def _skip_whitespace_and_comments(self) -> None:
        while True:
            ch = self._current()
            if ch is None:
                break
            if ch in " \t\n\r,":
                self._advance()
            elif ch == "\\" and self._peek() in ("\\", " ", "\t", None):
                # Line comment
                while self._current() is not None and self._current() != "\n":
                    self._advance()
            elif ch == "/" and self._peek() == "*":
                # Block comment
                self._advance()
                self._advance()
                while True:
                    if self._current() is None:
                        raise LexError(
                            f"Unterminated block comment at {self.line}:{self.col}"
                        )
                    if self._current() == "*" and self._peek() == "/":
                        self._advance()
                        self._advance()
                        break
                    self._advance()
            else:
                break

    def _read_number_after_first_digit(
        self, first_char: str, start_line: int, start_col: int
    ) -> Token:
        """Read a number after the first digit has been consumed."""
        num_str = first_char
        has_dot = first_char == "."

        while True:
            ch = self._current()
            if ch is None:
                break
            if ch.isdigit():
                num_str += self._advance()
            elif ch == "." and not has_dot:
                has_dot = True
                num_str += self._advance()
            else:
                break

        value = float(num_str) if has_dot else int(num_str)
        return Token("NUM", value, start_line, start_col)

    def _read_string(self) -> Token:
        """Read string literal after opening quote."""
        start_line, start_col = self.line, self.col
        self._advance()  # consume opening "
        chars = []
        escape_map = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"'}

        while True:
            ch = self._current()
            if ch is None:
                raise LexError(f"Unterminated string at {start_line}:{start_col}")
            if ch == '"':
                self._advance()
                break
            if ch == "\\":
                self._advance()
                esc = self._current()
                if esc is None:
                    raise LexError(f"Unterminated escape at {start_line}:{start_col}")
                chars.append(escape_map.get(esc, esc))
                self._advance()
            else:
                chars.append(self._advance())

        return Token("STRING", "".join(chars), start_line, start_col)

    def _read_char(self) -> Token:
        """Read character literal after opening '."""
        start_line, start_col = self.line, self.col
        self._advance()  # consume '
        ch = self._current()
        if ch is None:
            raise LexError(f"Unterminated char at {start_line}:{start_col}")

        if ch == "\\":
            self._advance()
            esc = self._current()
            if esc is None:
                raise LexError(f"Unterminated escape at {start_line}:{start_col}")
            escape_map = {"n": "\n", "t": "\t", "r": "\r", "'": "'", "\\": "\\"}
            value = escape_map.get(esc, esc)
            self._advance()
        else:
            value = ch
            self._advance()

        return Token("CHAR", ord(value), start_line, start_col)

    def tokenize(self) -> List[Token]:
        """Tokenize source and return token list."""
        tokens = []

        # Symbol mapping for single-char tokens
        symbols = {
            "@": ("DUP", "@"),
            "#": ("SWAP", "#"),
            "$": ("DROP", "$"),
            "+": ("ADD", "+"),
            "-": ("SUB", "-"),
            "*": ("MUL", "*"),
            "/": ("DIV", "/"),
            "%": ("MOD", "%"),
            "=": ("EQ", "="),
            "<": ("LT", "<"),
            ">": ("GT", ">"),
            "~": ("NOT", "~"),
            "|": ("PRINT", "|"),
            "^": ("PRINTC", "^"),
            "?": ("INPUT", "?"),
            ":": ("STORE", ":"),
            "[": ("WHILE", "["),
            "]": ("WEND", "]"),
            "(": ("CALL", "("),
            ")": ("ENDCALL", ")"),
            "{": ("BLOCK", "{"),
            "}": ("ENDB", "}"),
            ";": ("ELSE", ";"),
            "!": ("PUSH", "!"),
            "S": ("SIN", "S"),
            "C": ("COS", "C"),
            "T": ("TAN", "T"),
            "R": ("SQRT", "R"),
            "P": ("POW", "P"),
            "F": ("FLOOR", "F"),
            "L": ("LOG", "L"),
            "E": ("EXP", "E"),
            "M": ("ABS", "M"),
            "N": ("ATAN2", "N"),
            "W": ("RAND", "W"),
            "X": ("EXIT", "X"),
            "Z": ("TIME", "Z"),
            "&": ("ALEN", "&"),
            "A": ("ALOAD", "A"),
            "a": ("ASTORE", "a"),
            "_": ("AINIT", "_"),
            "U": ("USLEEP", "U"),
        }

        while True:
            self._skip_whitespace_and_comments()
            ch = self._current()

            if ch is None:
                tokens.append(Token("EOF", None, self.line, self.col))
                break

            start_line, start_col = self.line, self.col

            # String literal
            if ch == '"':
                tokens.append(self._read_string())
                continue

            # Character literal
            if ch == "'":
                tokens.append(self._read_char())
                continue

            # Push number: ! followed by digits
            if ch == "!":
                self._advance()
                self._skip_whitespace_and_comments()
                first = self._current()
                if first is None or not (
                    first.isdigit() or first == "-" or first == "."
                ):
                    raise LexError(
                        f"Expected number after '!' at {start_line}:{start_col}"
                    )
                if first == "-":
                    self._advance()
                    next_digit = self._current()
                    if next_digit is None or not next_digit.isdigit():
                        raise LexError(
                            f"Expected digit after '-' at {start_line}:{start_col}"
                        )
                    self._advance()
                    tok = self._read_number_after_first_digit(
                        next_digit, start_line, start_col
                    )
                    tok.value = -abs(tok.value)
                elif first == ".":
                    self._advance()
                    tok = self._read_number_after_first_digit(
                        first, start_line, start_col
                    )
                else:
                    self._advance()
                    tok = self._read_number_after_first_digit(
                        first, start_line, start_col
                    )
                tokens.append(tok)
                continue

            # Digits are variable indices (push var[n])
            if ch.isdigit():
                self._advance()
                tok = self._read_number_after_first_digit(ch, start_line, start_col)
                # Integers 0-99 are variable refs
                if isinstance(tok.value, int) and tok.value <= 99:
                    tok.kind = "VAR"
                tokens.append(tok)
                continue

            # Single-char symbols
            if ch in symbols:
                self._advance()
                kind, val = symbols[ch]
                tokens.append(Token(kind, val, start_line, start_col))
                continue

            raise LexError(f"Unexpected character {ch!r} at {start_line}:{start_col}")

        return tokens
