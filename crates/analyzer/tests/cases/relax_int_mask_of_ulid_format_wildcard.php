<?php

final class Ulid
{
    public const FORMAT_BASE32 = 1;
    public const FORMAT_RFC4122 = 2;
}

/**
 * @param int-mask-of<Ulid::FORMAT_*> $format
 */
function takes_format(int $format): void {}

function test_ulid_format_valid(): void
{
    takes_format(0);
    takes_format(Ulid::FORMAT_BASE32);
    takes_format(Ulid::FORMAT_RFC4122);
    takes_format(Ulid::FORMAT_BASE32 | Ulid::FORMAT_RFC4122);
}

function test_ulid_format_invalid(): void
{
    takes_format(8); // @mago-expect analysis:invalid-argument
}
