<?php

$a = [123, 0o123, 0x123, 123.456, '123', 1e12, 'string', [], null];
function list_to_csv(array $a): string
{
    $r = '';
    foreach ($a as $k) { // @mago-expect analysis:mixed-assignment
        if ($r) {
            $r .= ', ';
        }
        if (is_numeric($k)) {
            $r .= $k; // Invalid type `numeric` for right operand in string concatenation.
        } else {
            $r .= "'" . htmlspecialchars((string) $k, ENT_QUOTES) . "'";
        }
    }
    return $r;
}

echo list_to_csv($a);
