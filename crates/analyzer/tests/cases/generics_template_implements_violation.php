<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 * @mago-expect analysis:unused-template-parameter
 */
interface GenIntIface
{
}

/**
 * @mago-expect analysis:invalid-template-parameter
 *
 * @implements GenIntIface<string>
 */
final class GenStrImpl implements GenIntIface
{
}
