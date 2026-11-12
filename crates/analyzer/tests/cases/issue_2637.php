<?php

declare(strict_types=1);

namespace Issue2637;

final class Box
{
    public function same(Box $other): bool
    {
        return $other === $this;
    }
}

function loopWithRelatedElseif(int $n): void
{
    $prev = null;
    for ($i = 0; $i < $n; ++$i) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } elseif ($cur->same($prev)) {
            echo 'x';
        }
    }
}

function loopWithUnrelatedElseif(int $n): void
{
    $prev = null;
    for ($i = 0; $i < $n; ++$i) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } elseif ($i > 3) {
            echo 'x';
        }
    }
}

function loopUsesValueAfterElseif(int $n): void
{
    $prev = null;
    for ($i = 0; $i < $n; ++$i) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } elseif ($i > 3) {
            echo 'x';
        }

        $prev->same($cur);
    }
}

function loopWithoutElseif(int $n): void
{
    $prev = null;
    for ($i = 0; $i < $n; ++$i) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        }
    }
}

function loopWithElse(int $n): void
{
    $prev = null;
    for ($i = 0; $i < $n; ++$i) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } else {
            $cur->same($prev);
        }
    }
}

function whileLoop(int $n): void
{
    $prev = null;
    while ($n-- > 0) {
        $cur = new Box();
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } elseif ($n > 3) {
            echo 'x';
        }
    }
}

/** @param list<Box> $boxes */
function foreachLoop(array $boxes): void
{
    $prev = null;
    foreach ($boxes as $index => $cur) {
        if ($prev === null || $cur->same($prev)) {
            $prev = $cur;
        } elseif ($index > 3) {
            echo 'x';
        }
    }
}

function afterElseif(?Box $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === null || $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    }

    return $prev;
}

function afterElseifUsingValue(?Box $prev, Box $cur, int $i): Box
{
    if ($prev === null || $cur->same($prev)) {
        $prev = $cur;
    } elseif ($cur->same($prev)) {
        echo 'x';
    }

    return $prev;
}

function afterTwoElseifs(?Box $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === null || $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    } elseif ($i > 2) {
        echo 'y';
    }

    return $prev;
}

function afterElseifAndElse(?Box $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === null || $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    } else {
        echo 'y';
    }

    return $prev;
}

function afterReturningElseif(?Box $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === null || $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        return $cur;
    }

    return $prev;
}

function scalarAfterElseif(?int $prev, bool $flag, int $i): int
{
    if ($prev === null || $flag) {
        $prev = 5;
    } elseif ($i > 3) {
        echo 'x';
    }

    return $prev;
}

function scalarComparedAfterElseif(?int $prev, int $cur, int $i): int
{
    if ($prev === null || $cur === $prev) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    }

    return $prev;
}

function falseAfterElseif(Box|false $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === false || $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    }

    return $prev;
}

function elseifAssignsNullable(?int $prev, ?int $other, bool $flag, int $i): int
{
    if ($prev === null || $flag) {
        $prev = 5;
    } elseif ($i > 3) {
        $prev = $other;
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $prev;
}

function elseAssignsNullable(?int $prev, ?int $other, bool $flag, int $i): int
{
    if ($prev === null || $flag) {
        $prev = 5;
    } elseif ($i > 3) {
        echo 'x';
    } else {
        $prev = $other;
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $prev;
}

/** @return 5 */
function fallThroughStaysWide(?int $prev, bool $flag, int $i): int
{
    if ($prev === null || $flag) {
        $prev = 5;
    } elseif ($i > 3) {
        $prev = 5;
    }

    // @mago-expect analysis:invalid-return-statement
    return $prev;
}

function conjunctionStaysNullable(?Box $prev, Box $cur, bool $flag, int $i): Box
{
    if ($prev === null && $flag) {
        $prev = $cur;
    } elseif ($i > 3) {
        echo 'x';
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $prev;
}

function unassignedStaysNullable(?Box $prev, Box $cur, int $i): Box
{
    if ($prev === null || $cur->same($prev)) {
        echo 'y';
    } elseif ($i > 3) {
        echo 'x';
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $prev;
}
