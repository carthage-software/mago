<?php

declare(strict_types=1);

final readonly class Issue2110B
{
    public function __construct(
        private string $word,
        private int $value,
    ) {}

    public function word(): string
    {
        return $this->word;
    }

    public function value(): int
    {
        return $this->value;
    }

    public function withValue(int $value): Issue2110B
    {
        $vars = get_object_vars($this);
        $vars['value'] = $value;

        return new Issue2110B(...$vars);
    }

    /**
     * @return array{word: string, value: int}
     */
    public function toArray(): array
    {
        return get_object_vars($this);
    }
}

final class Issue2110Visibility
{
    public int $x = 1;
    protected int $y = 2;
    private int $z = 3;

    public function sum(): int
    {
        return $this->x + $this->y + $this->z;
    }

    /**
     * @return array{x: int, y: int, z: int, ...<array-key, mixed>}
     */
    public function inside(): array
    {
        return get_object_vars($this);
    }
}

/**
 * @return array{x: int, ...<array-key, mixed>}
 */
function issue2110Outside(Issue2110Visibility $object): array
{
    return get_object_vars($object);
}

class Issue2110Open
{
    public string $name = 'open';
}

/**
 * @return array{name: string, ...<array-key, mixed>}
 */
function issue2110NotFinal(Issue2110Open $object): array
{
    return get_object_vars($object);
}

/**
 * @return array<array-key, mixed>
 */
function issue2110Cast(): array
{
    return get_object_vars((object) [1 => 2]);
}
