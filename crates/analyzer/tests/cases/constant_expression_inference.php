<?php

declare(strict_types=1);

const CONST_EXPR_MAX = 5;
const CONST_EXPR_MIN = -CONST_EXPR_MAX;

enum ConstExprColor: int
{
    case Blue = 2;
}

final class ConstExpr
{
    public const MAX = 1;
    public const MIN = -self::MAX;
    public const SHIFTED = (1 << 4);
    public const LIST = [1, 2];
    public const INVERTED = ~1;
    // @mago-expect analysis:redundant-type-comparison
    public const ORDER = 7 <=> 3;
    // @mago-expect analysis:redundant-comparison
    public const SAME = 1 === 1;
    // @mago-expect analysis:redundant-null-coalesce
    public const FALLBACK = null ?? 5;
    public const SECOND = self::LIST[1];
    public const NAME = ConstExprColor::Blue->name;
}

function const_expr_takes_int(int $i): int
{
    return $i;
}

function const_expr_check(): string
{
    if (ConstExpr::MIN === -1) {
        echo 'min';
    }

    const_expr_takes_int(ConstExpr::SHIFTED);
    const_expr_takes_int(ConstExpr::INVERTED);
    const_expr_takes_int(ConstExpr::ORDER);
    const_expr_takes_int(ConstExpr::FALLBACK);
    const_expr_takes_int(ConstExpr::SECOND);

    return ConstExpr::NAME;
}
