<?php

function getByCountry(string $countryCode): LanguageEnum
{
    return match ($countryCode) {
        'PT'    => LanguageEnum::PT,
        'NL'    => LanguageEnum::NL,
        default => LanguageEnum::EN,
    };
}

function getLabel(Status $status): string
{
    return match ($status) {
        Status::Draft       => 'Draft',
        Status::NeedsReview => 'Needs review',
        Status::Published => 'Published',
        Status::Archived  => 'Archived',
    };
}

function getTypeIdentifier(string $doctrineType): ?TypeIdentifier
{
    return match ($doctrineType) {
        Types::SMALLINT, Types::INTEGER           => TypeIdentifier::INT,
        Types::FLOAT                              => TypeIdentifier::FLOAT,
        Types::BIGINT, Types::STRING, Types::TEXT => TypeIdentifier::STRING,
        default                                   => null,
    };
}

function getAlreadyDefault(string $countryCode): LanguageEnum
{
    return match ($countryCode) {
        'PT' => LanguageEnum::PT,
        'NL' => LanguageEnum::NL,
        default => LanguageEnum::EN,
    };
}
