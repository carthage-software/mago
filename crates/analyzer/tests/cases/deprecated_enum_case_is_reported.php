<?php

declare(strict_types=1);

enum DeprecatedCaseEnum
{
    case Current;

    /** @deprecated use Current */
    case Legacy;

    #[\Deprecated]
    case Old;
}

/**
 * @return list<DeprecatedCaseEnum>
 */
function deprecated_enum_cases(DeprecatedCaseEnum $value): array
{
    return [
        DeprecatedCaseEnum::Current,
        /** @mago-expect analysis:deprecated-class-constant */
        DeprecatedCaseEnum::Legacy,
        /** @mago-expect analysis:deprecated-class-constant */
        $value::Old,
    ];
}
