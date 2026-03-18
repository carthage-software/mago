<?php

declare(strict_types=1);

/** @param int<8, 8> $v */
function take_8(int $v): void {}

/** @param int<0, 0> $v */
function take_0(int $v): void {}

const SHIFT_OK = 1 << 3;
const SHIFT_OVER = 1 << 65;
const SHIFT_NEG = 1 >> -1;

take_8(SHIFT_OK);
take_0(SHIFT_OVER);
take_0(SHIFT_NEG); // @mago-expect analysis:possibly-invalid-argument
