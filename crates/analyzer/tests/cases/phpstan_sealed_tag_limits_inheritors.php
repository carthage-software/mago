<?php

declare(strict_types=1);

/**
 * @phpstan-sealed SealedTagFirst|SealedTagSecond
 */
abstract class SealedTagBase
{
}

final class SealedTagFirst extends SealedTagBase
{
}

final class SealedTagSecond extends SealedTagBase
{
}

/** @mago-expect analysis:invalid-extend */
final class SealedTagThird extends SealedTagBase
{
}
