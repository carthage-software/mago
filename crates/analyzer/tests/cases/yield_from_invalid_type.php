<?php

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-value-type
 */
function generator(): Generator
{
    yield from [1, 2, 3];
}
