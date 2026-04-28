<?php

declare(strict_types=1);

class GenAniBase2
{
}

class GenDogBase2 extends GenAniBase2
{
}

final class GenCarBase
{
}

/**
 * @template T of GenDogBase2
 *
 * @param T $val
 */
function gen_only_dogs2(GenDogBase2 $val): void
{
}

/** @mago-expect analysis:invalid-argument,template-constraint-violation */
gen_only_dogs2(new GenCarBase());
