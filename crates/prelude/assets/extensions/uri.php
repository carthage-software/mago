<?php

namespace Uri {
    class UriException extends \Exception
    {
    }

    class UriError extends \Error
    {
    }

    class InvalidUriException extends UriException
    {
    }

    enum UriComparisonMode
    {
        case IncludeFragment;
        case ExcludeFragment;
    }
}

namespace Uri\Rfc3986 {
    final readonly class Uri
    {
        public static function parse(string $uri, null|Uri $baseUrl = null): null|static
        {
        }

        public function __construct(string $uri, null|Uri $baseUrl = null) {}

        public function getScheme(): null|string
        {
        }

        public function getRawScheme(): null|string
        {
        }

        public function withScheme(null|string $scheme): static
        {
        }

        public function getUserInfo(): null|string
        {
        }

        public function getRawUserInfo(): null|string
        {
        }

        public function withUserInfo(null|string $userinfo): static
        {
        }

        public function getUsername(): null|string
        {
        }

        public function getRawUsername(): null|string
        {
        }

        public function getPassword(): null|string
        {
        }

        public function getRawPassword(): null|string
        {
        }

        public function getHost(): null|string
        {
        }

        public function getRawHost(): null|string
        {
        }

        public function withHost(null|string $host): static
        {
        }

        public function getPort(): null|int
        {
        }

        public function withPort(null|int $port): static
        {
        }

        public function getPath(): string
        {
        }

        public function getRawPath(): string
        {
        }

        public function withPath(string $path): static
        {
        }

        public function getQuery(): null|string
        {
        }

        public function getRawQuery(): null|string
        {
        }

        public function withQuery(null|string $query): static
        {
        }

        public function getFragment(): null|string
        {
        }

        public function getRawFragment(): null|string
        {
        }

        public function withFragment(null|string $fragment): static
        {
        }

        public function equals(
            Uri $uri,
            \Uri\UriComparisonMode $comparisonMode = \Uri\UriComparisonMode::ExcludeFragment,
        ): bool {
        }

        public function toString(): string
        {
        }

        public function toRawString(): string
        {
        }

        public function resolve(string $uri): static
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function __debugInfo(): array
        {
        }
    }
}

namespace Uri\WhatWg {
    class InvalidUrlException extends \Uri\InvalidUriException
    {
        public readonly array $errors;

        public function __construct(
            string $message = '',
            array $errors = [],
            int $code = 0,
            null|\Throwable $previous = null,
        ) {}
    }

    enum UrlValidationErrorType
    {
        case DomainToAscii;
        case DomainToUnicode;
        case DomainInvalidCodePoint;
        case HostInvalidCodePoint;
        case Ipv4EmptyPart;
        case Ipv4TooManyParts;
        case Ipv4NonNumericPart;
        case Ipv4NonDecimalPart;
        case Ipv4OutOfRangePart;
        case Ipv6Unclosed;
        case Ipv6InvalidCompression;
        case Ipv6TooManyPieces;
        case Ipv6MultipleCompression;
        case Ipv6InvalidCodePoint;
        case Ipv6TooFewPieces;
        case Ipv4InIpv6TooManyPieces;
        case Ipv4InIpv6InvalidCodePoint;
        case Ipv4InIpv6OutOfRangePart;
        case Ipv4InIpv6TooFewParts;
        case InvalidUrlUnit;
        case SpecialSchemeMissingFollowingSolidus;
        case MissingSchemeNonRelativeUrl;
        case InvalidReverseSoldius;
        case InvalidCredentials;
        case HostMissing;
        case PortOutOfRange;
        case PortInvalid;
        case FileInvalidWindowsDriveLetter;
        case FileInvalidWindowsDriveLetterHost;
    }

    final readonly class UrlValidationError
    {
        public readonly string $context;
        public readonly UrlValidationErrorType $type;
        public readonly bool $failure;

        public function __construct(string $context, UrlValidationErrorType $type, bool $failure) {}
    }

    final readonly class Url
    {
        /**
         * @param-out array<int, UrlValidationError> $errors
         */
        public static function parse(string $uri, null|Url $baseUrl = null, &$errors = null): null|static
        {
        }

        /**
         * @param-out array<int, UrlValidationError> $softErrors
         */
        public function __construct(string $uri, null|Url $baseUrl = null, &$softErrors = null) {}

        public function getScheme(): string
        {
        }

        public function withScheme(string $scheme): static
        {
        }

        public function getUsername(): null|string
        {
        }

        public function withUsername(null|string $username): static
        {
        }

        public function getPassword(): null|string
        {
        }

        public function withPassword(null|string $password): static
        {
        }

        public function getAsciiHost(): null|string
        {
        }

        public function getUnicodeHost(): null|string
        {
        }

        public function withHost(null|string $host): static
        {
        }

        public function getPort(): null|int
        {
        }

        public function withPort(null|int $port): static
        {
        }

        public function getPath(): string
        {
        }

        public function withPath(string $path): static
        {
        }

        public function getQuery(): null|string
        {
        }

        public function withQuery(null|string $query): static
        {
        }

        public function getFragment(): null|string
        {
        }

        public function withFragment(null|string $fragment): static
        {
        }

        public function equals(
            Url $url,
            \Uri\UriComparisonMode $comparisonMode = \Uri\UriComparisonMode::ExcludeFragment,
        ): bool {
        }

        public function toAsciiString(): string
        {
        }

        public function toUnicodeString(): string
        {
        }

        /**
         * @param-out array<int, UrlValidationError> $softErrors
         */
        public function resolve(string $uri, &$softErrors = null): static
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function __debugInfo(): array
        {
        }
    }
}
