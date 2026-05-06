<?php

declare(strict_types=1);

class SearchRequest
{
    public function __construct(
        public ?string $teamName = null,
        public ?string $countryCode = null,
        public ?string $role = null,
    ) {}

    /**
     * @return array{teamName?: string, countryCode?: string, role?: string}
     */
    public function getFilters(): array
    {
        return array_filter([
            'teamName' => $this->teamName,
            'countryCode' => $this->countryCode,
            'role' => $this->role,
        ]);
    }
}

/**
 * @return array{keep: string, keep_optional?: string}
 */
function drops_always_falsy_entries(string $x): array
{
    return array_filter([
        'drop' => null,
        'keep' => 'value',
        'keep_optional' => $x,
    ]);
}

/**
 * @param array<string, int|null> $values
 * @return array<string, int>
 */
function generic_array(array $values): array
{
    return array_filter($values);
}
