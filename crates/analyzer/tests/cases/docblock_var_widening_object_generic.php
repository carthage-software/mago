<?php

declare(strict_types=1);

/**
 * @template T
 */
final class BoxD
{
    /**
     * @param T $value
     */
    public function __construct(public readonly mixed $value) {}
}

/**
 * @return BoxD<int>
 */
function make_int_box(): BoxD
{
    return new BoxD(42);
}

function use_box_widen(): void
{
    $box = make_int_box();
    /** @var BoxD<int|string> $box */
    echo $box->value;
}

use_box_widen();
