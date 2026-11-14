<?php

declare(strict_types=1);

final class Settings
{
    public static int $instances = 0;

    public function __construct(
        public string $name,
    ) {}
}

/** @return properties-of<Settings> */
function to_array(Settings $settings): array
{
    return ['name' => $settings->name];
}
