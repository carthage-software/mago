<?php

/**
 * @return int[]|null
 */
function someFunction(): null|array
{
    return [1];
}

$b = someFunction() ?? throw new \Exception();

// This code is actually executed and running this script outputs `1`
echo "$b[0]" . PHP_EOL; // @mago-expect analysis:possibly-undefined-array-index
