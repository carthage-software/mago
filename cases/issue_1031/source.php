<?php

declare(strict_types=1);

interface ToolInterface
{
    public function getName(): string;
}

/**
 * @param array<non-negative-int, ToolInterface> $tools
 */
function filterTools(array $tools): void
{
    $filtered = array_filter($tools, function ($tool): bool {
        return $tool->getName() !== 'foo';
    });

    $filtered2 = array_filter($tools, fn($tool) => $tool->getName() !== 'foo');

    $filtered3 = array_filter(
        $tools,
        function ($key): bool {
            return $key > 0;
        },
        ARRAY_FILTER_USE_KEY,
    );

    $filtered4 = array_filter(
        $tools,
        function ($tool, $key): bool {
            return $tool->getName() !== 'foo' && $key > 0;
        },
        ARRAY_FILTER_USE_BOTH,
    );
}
