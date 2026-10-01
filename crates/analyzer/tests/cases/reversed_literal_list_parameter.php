<?php

declare(strict_types=1);

namespace ReversedLiteralListParameter;

/** @param list{string, string} $list */
function takesTwoList(array $list): void {}

// @mago-expect analysis:invalid-argument
takesTwoList([1 => 'x', 0 => 'y']);

$reversed = [1 => 'x', 0 => 'y'];
// @mago-expect analysis:invalid-argument
takesTwoList($reversed);

$maybeReversed = rand(0, 1) ? [1 => 'x', 0 => 'y'] : [0 => 'x', 1 => 'y'];
// @mago-expect analysis:invalid-argument
takesTwoList($maybeReversed);

takesTwoList([0 => 'x', 1 => 'y']);
takesTwoList(['x', 'y']);
