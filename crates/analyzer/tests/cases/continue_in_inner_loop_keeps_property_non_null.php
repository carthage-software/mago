<?php

declare(strict_types=1);

final class Branch
{
    /** @var list<string> */
    public array $hit;

    public function __construct()
    {
        $this->hit = [];
    }
}

/**
 * @param list<Branch> $branches
 *
 * @return list<string>
 */
function collect_hits(array $branches, bool $skip): array
{
    $hits = [];
    foreach ($branches as $branch) {
        foreach ([1, 2] as $_) {
            if ($skip) {
                continue;
            }

            if ($branch->hit !== []) {
                $hits = array_merge($hits, $branch->hit);
            }
        }
    }

    return $hits;
}
