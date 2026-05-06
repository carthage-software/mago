<?php

declare(strict_types=1);

/**
 * @param list<array{fileId: int, pageNumber: int}> $pages
 *
 * @return list<array{fileId: int, pageNumbers: list<int>}>
 */
function groupConsecutivePages(array $pages): array
{
    $groups = [];

    foreach ($pages as $page) {
        $last = \array_key_last($groups);

        if (null !== $last && $groups[$last]['fileId'] === $page['fileId']) {
            $groups[$last]['pageNumbers'][] = $page['pageNumber'];
        } else {
            $groups[] = [
                'fileId' => $page['fileId'],
                'pageNumbers' => [$page['pageNumber']],
            ];
        }
    }

    return $groups;
}
