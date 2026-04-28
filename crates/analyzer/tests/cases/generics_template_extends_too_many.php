<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
class GenSingleParam
{
}

/**
 * @mago-expect analysis:excess-template-parameter
 *
 * @extends GenSingleParam<int, string>
 */
final class GenSingleChildExcess extends GenSingleParam
{
}
