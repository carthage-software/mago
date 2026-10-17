<?php

declare(strict_types=1);

final class Issue2504CallbackException extends Exception {}

final class Issue2504Mapper
{
    /** @throws Issue2504CallbackException */
    public function fromRow(int $row): int
    {
        throw new Issue2504CallbackException();
    }

    /** @throws Issue2504CallbackException */
    public static function pregCallback(array $matches): string
    {
        throw new Issue2504CallbackException();
    }
}

/** @throws Issue2504CallbackException */
function issue_2504_named_callback(int $row): int
{
    throw new Issue2504CallbackException();
}

/** @throws Issue2504CallbackException */
function issue_2504_preg_callback(array $matches): string
{
    throw new Issue2504CallbackException();
}

final class Issue2504Repository
{
    /** @throws Issue2504CallbackException */
    public function mapRows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_map(static fn(int $row): int => $mapper->fromRow($row), $rows);
    }

    /** @throws Issue2504CallbackException */
    public function filterRows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_filter(
            $rows,
            callback: static fn(int $row): bool => $mapper->fromRow($row) > 0,
        );
    }

    /** @throws Issue2504CallbackException */
    public function compareRows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_udiff(
            $rows,
            $rows,
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @throws Issue2504CallbackException */
    public function compareRowsWithUnpackedArrays(array $rows, array $otherRows, Issue2504Mapper $mapper): array
    {
        return ArRaY_UdIfF(
            $rows,
            ...[$otherRows],
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function compareRowsWithUnpackedArraysWithoutThrows(array $rows, array $otherRows, Issue2504Mapper $mapper): array
    {
        return ArRaY_UdIfF(
            $rows,
            ...[$otherRows],
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @throws Issue2504CallbackException */
    public function compareRowsAndKeys(array $rows, Issue2504Mapper $mapper): array
    {
        return array_udiff_uassoc(
            $rows,
            $rows,
            static fn(int $left, int $right): int => $mapper->fromRow($left),
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @throws Issue2504CallbackException */
    public function compareRowsAndKeysWithUnpackedArrays(array $rows, array $otherRows, Issue2504Mapper $mapper): array
    {
        return ArRaY_UdIfF_UaSsOc(
            $rows,
            ...[$otherRows],
            static fn(int $left, int $right): int => $mapper->fromRow($left),
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function compareRowsWithoutThrows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_udiff_uassoc(
            $rows,
            $rows,
            static fn(int $left, int $right): int => $mapper->fromRow($left),
            static fn(int $left, int $right): int => $mapper->fromRow($left),
        );
    }

    /** @throws Issue2504CallbackException */
    public function replaceRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                '/\\d+/' => static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
            ],
            $subject,
        );
    }

    /** @throws Issue2504CallbackException */
    public function replaceParenthesizedRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                '/\\d+/' => (static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0))),
            ],
            $subject,
        );
    }

    /** @throws Issue2504CallbackException */
    public function replaceSpreadRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                ...[
                    '/\\d+/' => static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
                ],
            ],
            $subject,
        );
    }

    /** @throws Issue2504CallbackException */
    public function replaceLegacyRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            array(
                '/\\d+/' => static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
            ),
            $subject,
        );
    }

    /** @mago-expect analysis:duplicate-array-key */
    public function replaceOverriddenRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                '/\\d+/' => /** @throws Issue2504CallbackException */ static fn(array $matches): string =>
                    (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
                '/\\d+/' => static fn(array $matches): string => '',
            ],
            $subject,
        );
    }

    public function replaceSpreadOverriddenRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                ...[
                    '/\\d+/' => /** @throws Issue2504CallbackException */ static fn(array $matches): string =>
                        (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
                ],
                '/\\d+/' => static fn(array $matches): string => '',
            ],
            $subject,
        );
    }

    /** @throws Issue2504CallbackException */
    public function replaceStoredRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        $callbacks = [
            '/\\d+/' => static function (array $matches) use ($mapper): string {
                return (string) $mapper->fromRow((int) ($matches[0] ?? 0));
            },
        ];

        return preg_replace_callback_array($callbacks, $subject);
    }

    /** @throws Issue2504CallbackException */
    public function replaceStoredRowsWithNamedPattern(string $subject, Issue2504Mapper $mapper): ?string
    {
        $callbacks = [
            '/\\d+/' => static function (array $matches) use ($mapper): string {
                return (string) $mapper->fromRow((int) ($matches[0] ?? 0));
            },
        ];

        return preg_replace_callback_array(pattern: $callbacks, subject: $subject);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceStoredRowsWithoutThrows(string $subject, Issue2504Mapper $mapper): ?string
    {
        $callbacks = [
            '/\\d+/' => static function (array $matches) use ($mapper): string {
                return (string) $mapper->fromRow((int) ($matches[0] ?? 0));
            },
        ];

        return preg_replace_callback_array($callbacks, $subject);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceStoredRowsTwiceWithoutThrows(string $subject, Issue2504Mapper $mapper): void
    {
        $callbacks = [
            '/\\d+/' => static function (array $matches) use ($mapper): string {
                return (string) $mapper->fromRow((int) ($matches[0] ?? 0));
            },
        ];

        try {
            preg_replace_callback_array($callbacks, $subject);
        } catch (Issue2504CallbackException) {
        }

        preg_replace_callback_array($callbacks, $subject);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceRowsWithStringCallbackWithoutThrows(string $subject): ?string
    {
        return preg_replace_callback_array(
            ['/\\d+/' => 'issue_2504_preg_callback'],
            $subject,
        );
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceRowsWithCallableArrayWithoutThrows(string $subject): ?string
    {
        return preg_replace_callback_array(
            ['/\\d+/' => [Issue2504Mapper::class, 'pregCallback']],
            $subject,
        );
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceRowsWithoutThrows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                '/\\d+/' => static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
            ],
            $subject,
        );
    }

    /** @throws Issue2504CallbackException */
    public function mapRowsWithNamedCallback(array $rows): array
    {
        return array_map('issue_2504_named_callback', $rows);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function mapRowsWithoutThrows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_map(static fn(int $row): int => $mapper->fromRow($row), $rows);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function registerDeferredCallback(Issue2504Mapper $mapper): void
    {
        register_shutdown_function(static function () use ($mapper): void {
            $mapper->fromRow(1);
        });
    }
}
