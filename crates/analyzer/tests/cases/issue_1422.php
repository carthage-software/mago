<?php

declare(strict_types=1);

/**
 * @template T of array
 */
class Box
{
    /**
     * @param T $value
     */
    public function __construct(
        /** @mago-expect analysis:unused-property */
        private array $value = [],
    ) {}
}

/** @param Box<array<string, mixed>> $box */
function accept(Box $box): void {}

/** @mago-expect analysis:invalid-argument */
accept(new Box([]));
/** @mago-expect analysis:invalid-argument */
accept(new Box());

/** @var Box<array<string, mixed>> */
$box = new Box();
accept($box);
