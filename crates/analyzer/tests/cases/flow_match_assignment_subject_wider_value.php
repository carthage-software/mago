<?php

declare(strict_types=1);

enum FlowMatchWiderValueKind: string
{
    case A = 'a';
    case B = 'b';
}

final class FlowMatchWiderValueDocblock
{
    /** @var 'GET'|'POST' */
    public string $method = 'GET';

    /** @var FlowMatchWiderValueKind */
    public $kind = FlowMatchWiderValueKind::A;

    /** @var int<0, 2> */
    public int $level = 0;

    /** @var true */
    public bool $enabled = true;

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function literal(string $m): int
    {
        return match ($this->method = strtoupper($m)) {
            'GET' => 1,
            'POST' => 2,
            'PUT' => 3,
            default => 0,
        };
    }

    /**
     * @throws UnhandledMatchError
     *
     * @mago-expect analysis:invalid-property-assignment-value
     * @mago-expect analysis:match-not-exhaustive
     */
    public function literalWithoutDefault(string $m): int
    {
        return match ($this->method = strtoupper($m)) {
            'GET' => 1,
            'POST' => 2,
        };
    }

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function enumOrString(FlowMatchWiderValueKind|string $k): int
    {
        return match ($this->kind = $k) {
            FlowMatchWiderValueKind::A => 1,
            FlowMatchWiderValueKind::B => 2,
            'legacy' => 3,
            default => 0,
        };
    }

    /**
     * @mago-expect analysis:mixed-property-type-coercion
     */
    public function mixedValue(mixed $m): int
    {
        return match ($this->kind = $m) {
            FlowMatchWiderValueKind::A => 1,
            FlowMatchWiderValueKind::B => 2,
            default => 0,
        };
    }

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function range(int $i): string
    {
        return match ($this->level = $i) {
            0 => 'zero',
            1 => 'one',
            2 => 'two',
            default => 'other',
        };
    }

    /**
     * @mago-expect analysis:property-type-coercion
     */
    public function boolean(bool $b): int
    {
        return match ($this->enabled = $b) {
            true => 1,
            false => 0,
        };
    }

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function compound(int $i): int
    {
        return match ($this->level += $i) {
            0 => 1,
            1, 2 => 2,
            default => 0,
        };
    }
}

final class FlowMatchWiderValueStatic
{
    /** @var 'a'|'b' */
    public static string $tag = 'a';

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public static function withDefault(string $s): int
    {
        return match (self::$tag = $s) {
            'a' => 1,
            'b' => 2,
            default => 0,
        };
    }

    /**
     * @throws UnhandledMatchError
     *
     * @mago-expect analysis:invalid-property-assignment-value
     * @mago-expect analysis:match-not-exhaustive
     */
    public static function withoutDefault(string $s): int
    {
        return match (self::$tag = $s) {
            'a' => 1,
            'b' => 2,
        };
    }
}

final class FlowMatchWiderValueHooked
{
    public FlowMatchWiderValueKind $kind = FlowMatchWiderValueKind::A {
        set(FlowMatchWiderValueKind|string $value) {
            $this->kind = $value instanceof FlowMatchWiderValueKind
                ? $value
                : (FlowMatchWiderValueKind::tryFrom($value) ?? FlowMatchWiderValueKind::A);
        }
    }

    public function withDefault(FlowMatchWiderValueKind|string $input): int
    {
        return match ($this->kind = $input) {
            FlowMatchWiderValueKind::A => 1,
            FlowMatchWiderValueKind::B => 2,
            'legacy' => 3,
            default => 0,
        };
    }

    /**
     * @throws UnhandledMatchError
     *
     * @mago-expect analysis:match-not-exhaustive
     */
    public function withoutDefault(FlowMatchWiderValueKind|string $input): int
    {
        return match ($this->kind = $input) {
            FlowMatchWiderValueKind::A => 1,
            FlowMatchWiderValueKind::B => 2,
        };
    }
}

/**
 * @property 'x'|'y' $magic
 * @property-read 'x'|'y' $flag
 * @property-write string $flag
 */
final class FlowMatchWiderValueMagic
{
    /** @var array<string, mixed> */
    private array $data = [];

    public function __get(string $name): mixed
    {
        return $this->data[$name] ?? null;
    }

    public function __set(string $name, mixed $value): void
    {
        $this->data[$name] = $value;
    }

    /**
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function magic(string $s): int
    {
        return match ($this->magic = $s) {
            'x' => 1,
            'y' => 2,
            'z' => 3,
            default => 0,
        };
    }

    public function readWrite(string $s): int
    {
        return match ($this->flag = $s) {
            'x' => 1,
            'y' => 2,
            'z' => 3,
            default => 0,
        };
    }

    /**
     * @throws UnhandledMatchError
     *
     * @mago-expect analysis:match-not-exhaustive
     */
    public function readWriteWithoutDefault(string $s): int
    {
        return match ($this->flag = $s) {
            'x' => 1,
            'y' => 2,
        };
    }
}

function flow_match_assignment_subject_wider_value(
    FlowMatchWiderValueHooked $hooked,
    FlowMatchWiderValueKind|string $input,
): int {
    return match ($hooked->kind = $input) {
        FlowMatchWiderValueKind::A => 1,
        FlowMatchWiderValueKind::B => 2,
        'legacy' => 3,
        default => 0,
    };
}
