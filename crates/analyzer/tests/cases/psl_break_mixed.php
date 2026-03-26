<?php

declare(strict_types=1);

function test_break_in_loop_preserves_type(): void
{
    $bitsLeft = 0;
    $buffer = 0;
    $result = '';
    for ($i = 0; $i < 10; $i++) {
        if ($i === 5) {
            break;
        }

        $buffer = ($buffer << 5) | $i;
        $bitsLeft += 5;

        if ($bitsLeft >= 8) {
            $bitsLeft -= 8;
            $result .= chr(($buffer >> $bitsLeft) & 0xFF);
        }
    }
}
