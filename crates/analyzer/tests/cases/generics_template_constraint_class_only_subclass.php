<?php

declare(strict_types=1);

class GenAniBase
{
}

class GenDogBase extends GenAniBase
{
}

class GenPugBase extends GenDogBase
{
}

/**
 * @template T of GenDogBase
 *
 * @param T $val
 *
 * @return T
 */
function gen_only_dogs(GenDogBase $val): GenDogBase
{
    return $val;
}

gen_only_dogs(new GenPugBase());
