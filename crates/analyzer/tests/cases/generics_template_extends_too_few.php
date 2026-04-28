<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 *
 * @mago-expect analysis:unused-template-parameter(2)
 */
class GenTwoParams
{
}

/**
 * @mago-expect analysis:missing-template-parameter
 *
 * @extends GenTwoParams<int>
 */
final class GenTwoChildFew extends GenTwoParams
{
}
