<?php

function getByCountry(string $countryCode): LanguageEnum
{
    return match ($countryCode) {
        'PT' => LanguageEnum::PT,
        'NL' => LanguageEnum::NL,
        'ES' => LanguageEnum::ES,
        'IT' => LanguageEnum::IT,
        'DE' => LanguageEnum::DE,
        'FR' => LanguageEnum::FR,
        default => LanguageEnum::EN,
    };
}

function getLabel(Status $status): string
{
    return match ($status) {
        Status::Draft => 'Draft',
        Status::NeedsReview => 'Needs review',
        // Published states are grouped separately.
        Status::Published => 'Published',
        Status::Archived => 'Archived',
    };
}

function getAlreadyAligned(string $countryCode): LanguageEnum
{
    return match ($countryCode) {
        'PT'    => LanguageEnum::PT,
        'NL'    => LanguageEnum::NL,
        'ES'    => LanguageEnum::ES,
        default => LanguageEnum::EN,
    };
}

function getTypeIdentifier(string $doctrineType): ?TypeIdentifier
{
    return match ($doctrineType) {
        Types::SMALLINT, Types::INTEGER => TypeIdentifier::INT,
        Types::FLOAT => TypeIdentifier::FLOAT,
        Types::BIGINT, Types::STRING, Types::TEXT => TypeIdentifier::STRING,
        default => null,
    };
}

function getSingleArm(Status $status): string
{
    return match ($status) {
        Status::Draft => 'Draft',
    };
}
