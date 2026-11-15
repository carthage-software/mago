<?php

declare(strict_types=1);

enum StringEnum: string
{
    case Name = 'Value';
    case Other = 'Second';

    /**
     * @return array<string>
     */
    public static function toArray(): array
    {
        return array_column(self::cases(), 'value');
    }

    /**
     * @return list<'Value'|'Second'>
     */
    public static function values(): array
    {
        return array_column(self::cases(), 'value');
    }

    /**
     * @return array<'Value'|'Second', 'Name'|'Other'>
     */
    public static function namesByValue(): array
    {
        return array_column(self::cases(), 'name', 'value');
    }

    /**
     * @return array<'Name'|'Other', StringEnum>
     */
    public static function casesByName(): array
    {
        return array_column(self::cases(), null, 'name');
    }
}

enum IntEnum: int
{
    case One = 1;
    case Two = 2;

    /**
     * @return list<1|2>
     */
    public static function values(): array
    {
        return array_column(self::cases(), 'value');
    }
}

enum Suit
{
    case First;
    case Second;

    /**
     * @return list<'First'|'Second'>
     */
    public static function names(): array
    {
        return array_column(self::cases(), 'name');
    }
}

/**
 * @param list<StringEnum::Name|IntEnum::Two> $cases
 *
 * @return list<'Value'|2>
 */
function mixed_case_values(array $cases): array
{
    return array_column($cases, 'value');
}
