<?php

declare(strict_types=1);

final class ArrayOrTypedReview
{
}

interface ArrayOrTypedRepository
{
    /** @return ArrayOrTypedReview[] */
    public function byProduct(int $productId): array;
}

final class SqlArrayOrTypedRepository implements ArrayOrTypedRepository
{
    /**
     * Legacy PhpStorm idiom `array|Review[]` for "array of Review".
     *
     * @return array|ArrayOrTypedReview[]
     */
    public function byProduct(int $productId): array
    {
        return [new ArrayOrTypedReview()];
    }
}

final class ArrayOrTypedPrinter
{
    /** @param ArrayOrTypedReview[] $reviews */
    public function print(array $reviews): int
    {
        return count($reviews);
    }

    public function run(SqlArrayOrTypedRepository $repository): int
    {
        return $this->print($repository->byProduct(1));
    }
}

/**
 * General unions are not narrowed: `array` still accepts the empty array,
 * so this must stay silent (contrast the `array|Foo[]` shorthand above).
 *
 * @return array|non-empty-list<string>
 */
function maybeNames(): array
{
    return [];
}

echo (new ArrayOrTypedPrinter())->run(new SqlArrayOrTypedRepository());
