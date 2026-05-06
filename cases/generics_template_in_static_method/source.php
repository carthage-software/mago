<?php

declare(strict_types=1);

final class GenStMethCtor
{
    /**
     * @template T
     *
     * @param T $value
     *
     * @return list<T>
     */
    public static function repeat3(mixed $value): array
    {
        return [$value, $value, $value];
    }
}

$nums = GenStMethCtor::repeat3(7);
foreach ($nums as $n) {
    echo $n + 1;
}
