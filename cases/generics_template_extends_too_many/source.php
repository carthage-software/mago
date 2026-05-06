<?php

declare(strict_types=1);

/**
 * @template T
 *
 */
class GenSingleParam {}

/**
 *
 * @extends GenSingleParam<int, string>
 */
final class GenSingleChildExcess extends GenSingleParam {}
