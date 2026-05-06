<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 */
interface GenIntIface {}

/**
 *
 * @implements GenIntIface<string>
 */
final class GenStrImpl implements GenIntIface {}
