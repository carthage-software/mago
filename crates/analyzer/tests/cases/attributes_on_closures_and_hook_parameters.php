<?php

declare(strict_types=1);

#[Attribute(Attribute::TARGET_CLASS_CONSTANT | Attribute::TARGET_FUNCTION | Attribute::IS_REPEATABLE)]
final class OnConst
{
    public function __construct(public int $n = 0) {}
}

#[Attribute(Attribute::TARGET_CLASS_CONSTANT)]
final class OnlyConst {}

function t(): int
{
    // @mago-expect analysis:non-existent-attribute-class
    $a = #[\Nope] fn(): int => 1;
    // @mago-expect analysis:non-existent-attribute-class
    $b = #[\Nope] function (): int { return 1; };
    // @mago-expect analysis:invalid-attribute-target
    $c = #[OnlyConst] static fn(): int => 1;
    // @mago-expect analysis:invalid-argument
    $d = #[OnConst('x')] static fn(): int => 1;
    $e = #[OnConst(1)] static fn(): int => 1;

    return $a() + $b() + $c() + $d() + $e();
}

final class Hooks
{
    public int $p = 0 {
        set(
            // @mago-expect analysis:non-existent-attribute-class
            #[\Nope]
            int $value,
        ) {
            $this->p = $value;
        }
    }
}
