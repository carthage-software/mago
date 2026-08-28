<?php

declare(strict_types=1);

interface ProbeTarget {}

function accepts_plain_strings(string $s): void
{
    if (is_a($s, ProbeTarget::class, true)) {
        echo 'a';
    }

    if (is_subclass_of($s, ProbeTarget::class)) {
        echo 'b';
    }

    if (method_exists($s, 'probe')) {
        echo 'c';
    }
}

function knows_the_result_when_strings_are_disallowed(string $s): void
{
    /** @mago-expect analysis:impossible-condition */
    if (is_a($s, ProbeTarget::class)) {
        echo 'a';
    }

    /** @mago-expect analysis:impossible-condition */
    if (is_subclass_of($s, ProbeTarget::class, false)) {
        echo 'b';
    }
}

function keeps_objects_undecided(object $o): void
{
    if (is_a($o, ProbeTarget::class)) {
        echo 'a';
    }

    if (is_subclass_of($o, ProbeTarget::class, false)) {
        echo 'b';
    }
}

function keeps_unions_and_unknown_flags_undecided(object|string $os, bool $allow): void
{
    if (is_a($os, ProbeTarget::class)) {
        echo 'a';
    }

    if (is_a($os, ProbeTarget::class, $allow)) {
        echo 'b';
    }
}
