<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
interface GenIfaceSingle
{
}

/**
 * @mago-expect analysis:excess-template-parameter
 *
 * @implements GenIfaceSingle<int, string>
 */
final class GenIfaceSingleImpl implements GenIfaceSingle
{
}
