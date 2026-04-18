<?php

const RELAX_FLAG_ALPHA = 1;

const RELAX_FLAG_BETA = 2;

const RELAX_FLAG_GAMMA = 4;

/**
 * @param int-mask-of<RELAX_FLAG_*> $flags
 */
function takes_flags(int $flags): void {}

function test_global_wildcard_valid(): void
{
    takes_flags(0);
    takes_flags(RELAX_FLAG_ALPHA);
    takes_flags(RELAX_FLAG_BETA);
    takes_flags(RELAX_FLAG_GAMMA);
    takes_flags(RELAX_FLAG_ALPHA | RELAX_FLAG_GAMMA);
    takes_flags(RELAX_FLAG_ALPHA | RELAX_FLAG_BETA | RELAX_FLAG_GAMMA);
}

function test_global_wildcard_invalid(): void
{
    takes_flags(16); // @mago-expect analysis:invalid-argument
}
