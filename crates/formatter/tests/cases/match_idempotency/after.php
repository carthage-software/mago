<?php

class DoctrineExtractor
{
    /**
     * Gets the corresponding built-in PHP type.
     */
    private function getTypeIdentifier(string $doctrineType): null|TypeIdentifier
    {
        return match ($doctrineType) {
            Types::SMALLINT, Types::INTEGER => TypeIdentifier::INT,
            Types::FLOAT => TypeIdentifier::FLOAT,
            Types::BIGINT, Types::STRING, Types::TEXT, Types::GUID, Types::DECIMAL => TypeIdentifier::STRING,
            Types::BOOLEAN => TypeIdentifier::BOOL,
            Types::BLOB, Types::BINARY => TypeIdentifier::RESOURCE,
            'object', // DBAL < 4
            Types::DATE_MUTABLE,
            Types::DATETIME_MUTABLE,
            Types::DATETIMETZ_MUTABLE,
            'vardatetime',
            Types::TIME_MUTABLE,
            Types::DATE_IMMUTABLE,
            Types::DATETIME_IMMUTABLE,
            Types::DATETIMETZ_IMMUTABLE,
            Types::TIME_IMMUTABLE,
            Types::DATEINTERVAL,
                => TypeIdentifier::OBJECT,
            'array', // DBAL < 4
            'json_array', // DBAL < 3
            Types::SIMPLE_ARRAY,
                => TypeIdentifier::ARRAY,
            default => null,
        };
    }
}
