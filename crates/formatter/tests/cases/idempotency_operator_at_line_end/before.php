<?php

if (0 === $codePoint // NULL
    || (0x200B <= $codePoint && 0x200F >= $codePoint) // ZERO WIDTH SPACE to RIGHT-TO-LEFT MARK
    || 0x2028 === $codePoint // LINE SEPARATOR
    || (0x2060 <= $codePoint && 0x2063 >= $codePoint) // WORD JOINER to INVISIBLE SEPARATOR
) {
    continue;
}
