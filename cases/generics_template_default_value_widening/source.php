<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 */
final class GenIntCnst {}

function take_int_cnst(GenIntCnst $g): void {}

take_int_cnst(new GenIntCnst());
