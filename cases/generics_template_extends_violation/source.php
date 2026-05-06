<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 */
class GenIntCnstParent {}

/**
 *
 * @extends GenIntCnstParent<string>
 */
final class GenStrChild extends GenIntCnstParent {}
