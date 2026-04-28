<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 * @mago-expect analysis:unused-template-parameter
 */
class GenIntCnstParent
{
}

/**
 * @mago-expect analysis:invalid-template-parameter
 *
 * @extends GenIntCnstParent<string>
 */
final class GenStrChild extends GenIntCnstParent
{
}
