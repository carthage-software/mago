<?php

declare(strict_types=1);

interface EnumTypeInterface {}

enum EnumType: string implements EnumTypeInterface
{
    case A = 'a';
    case B = 'b';
}

class ZClass
{
    /**
     * @var list<EnumTypeInterface>
     */
    public array $enumCases = [];

    /**
     * @param class-string<EnumTypeInterface&BackedEnum> $backedEnum
     *
     * @mago-expect analysis:possibly-static-access-on-interface
     */
    public function useBackedEnum(string $backedEnum): void
    {
        foreach ($backedEnum::cases() as $case) {
            $this->enumCases[] = $case;
        }
    }
}

$z = new ZClass();
$z->useBackedEnum(EnumType::class);

var_dump($z->enumCases);
