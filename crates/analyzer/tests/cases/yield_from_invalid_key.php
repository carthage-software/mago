<?php

/**
 * @return Generator<string, string>
 *
 * @mago-expect analysis:yield-from-invalid-key-type
 */
function generator(): Generator
{
    yield from [1 => 'value'];
}
