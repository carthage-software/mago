<?php

declare(strict_types=1);

function test_session_mutation(): void
{
    $_SESSION['user_id'] = 1;
}

$_SESSION['user_id'] = 0;

test_session_mutation();

// @mago-expect analysis:mixed-operand
if (isset($_SESSION['user_id']) && $_SESSION['user_id']) {
    echo 'some user';
}

$a = 1;

function mutate_global(): void
{
    global $a;
    $a = 2;
}

mutate_global();

if ($a !== 1) {
    echo 'a is not 1';
}
