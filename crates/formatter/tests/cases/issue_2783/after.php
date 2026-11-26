<?php

return $this->sort($query)
    ->flatMap(
        fn($sortedQuery) => $this->paginateDistinct($paging, $sortedQuery, 'np.' . self::DB_PK, $rowCondition, $more),
    );

return $this->sort($query)
    ?->flatMap(
        fn($sortedQuery) => $this->paginateDistinct($paging, $sortedQuery, 'np.' . self::DB_PK, $rowCondition, $more),
    );

final class Foo
{
    public function bar()
    {
        return $this->sort($query)
            ->flatMap(
                fn($sortedQuery) => $this->paginateDistinct(
                    $paging,
                    $sortedQuery,
                    'np.' . self::DB_PK,
                    $rowCondition,
                    $more,
                ),
            );
    }
}

return $this->sort($query)
    ->flatMap(fn($sortedQuery) => $this->paginateDistinct($a, $b));
