<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 *
 * @mago-expect analysis:unused-template-parameter(2)
 */
interface GenIfaceTwo
{
}

/**
 * @mago-expect analysis:missing-template-parameter
 *
 * @implements GenIfaceTwo<int>
 */
final class GenIfaceTwoImpl implements GenIfaceTwo
{
}
