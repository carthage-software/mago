<?php

declare(strict_types=1);

/**
 * @property string|int $a
 * @property ?string $b
 */
#[AllowDynamicProperties]
final class Issue2177DynamicOk
{
    public function foo(): void
    {
        switch (true) {
            case isset($this->a):
            case is_int($this->a):
                echo 'a';

            case isset($this->b):
                echo 'b';
        }
    }
}

/**
 * @property string|int $a
 * @property string $b
 */
#[AllowDynamicProperties]
final class Issue2177DynamicNotOk
{
    public function foo(): void
    {
        switch (true) {
            case isset($this->a):
            case is_int($this->a):
                echo 'a';

            case isset($this->b):
                echo 'b';
        }
    }
}

/**
 * @property string $b
 */
final class Issue2177WithoutAttribute
{
    public function foo(): void
    {
        if (isset($this->b)) {
            echo $this->b;
        }
    }
}

/**
 * @property string $b
 */
final class Issue2177Narrowing
{
    public function foo(): string
    {
        if (isset($this->b)) {
            return $this->b;
        }

        return '';
    }

    public function bar(): string
    {
        return $this->b;
    }
}

final class Issue2177RealProperty
{
    public string $withDefault = 'x';

    public ?string $nullable = null;

    /**
     * @mago-expect analysis:redundant-condition
     */
    public function alwaysSet(): void
    {
        if (isset($this->withDefault)) {
            echo 'set';
        }
    }

    public function maybeNull(): void
    {
        if (isset($this->nullable)) {
            echo 'set';
        }
    }
}
