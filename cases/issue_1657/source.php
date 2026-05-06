<?php

declare(strict_types=1);

enum Issue1657Color: string
{
    case Red = 'red';
    case Blue = 'blue';

    /** @var array<value-of<self>, int> */
    private const array WEIGHTS = [
        self::Red->value => 1,
        self::Blue->value => 2,
    ];

    /** @var value-of<self> */
    private const string DEFAULT_NAME = 'red';

    /** @return array<value-of<self>, int> */
    public function getWeights(): array
    {
        return self::WEIGHTS;
    }
}

class Issue1657Registry
{
    /** @var array<value-of<Issue1657Color>, int> */
    private const array WEIGHTS = [
        Issue1657Color::Red->value => 1,
        Issue1657Color::Blue->value => 2,
    ];

    /** @var value-of<Issue1657Color> */
    private const string DEFAULT = 'red';

    /** @var array<value-of<Issue1657Color>, int> */
    private array $weights = [
        Issue1657Color::Red->value => 1,
        Issue1657Color::Blue->value => 2,
    ];

    /** @var value-of<Issue1657Color> */
    private string $current = 'blue';

    /** @return array<value-of<Issue1657Color>, int> */
    public function getWeights(): array
    {
        return $this->weights + self::WEIGHTS;
    }

    /** @return value-of<Issue1657Color> */
    public function getCurrent(): string
    {
        return $this->current;
    }
}

/**
 * @param value-of<Issue1657Color> $color
 * @param array<value-of<Issue1657Color>, int> $extras
 */
function issue_1657_consume(
    string $color = 'red',
    array $extras = [
        Issue1657Color::Red->value => 1,
        Issue1657Color::Blue->value => 2,
    ],
): void {
    echo $color;
    echo count($extras);
}

/** @var array<value-of<Issue1657Color>, int> $module_level */
$module_level = [
    Issue1657Color::Red->value => 1,
    Issue1657Color::Blue->value => 2,
];
