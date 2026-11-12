<?php

final class ProgressIndicator
{
    private const FORMATS = [
        'normal' => ' %indicator% %message%',
        'normal_no_ansi' => ' %message%',

        'verbose' => ' %indicator% %message% (%elapsed:6s%)',
        'verbose_no_ansi' => ' %message% (%elapsed:6s%)',
    ];
}

$x = ['a' => 1,

    'b' => 2];

$y = [1, 2, 3];

foo($a,

    $b, $c);
