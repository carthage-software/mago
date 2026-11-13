<?php

declare(strict_types=1);

namespace Issue2618;

use PDO;
use PDOException;
use PDOStatement;
use RuntimeException;

final class Box
{
    public int $value = 0;

    public function get(): int
    {
        return $this->value;
    }
}

/**
 * @ignore-falsable-return
 */
function lenient(): Box|false
{
    return new Box();
}

function strict(): Box|false
{
    return new Box();
}

/**
 * @throws PDOException
 */
function count_users(PDO $pdo): mixed
{
    $statement = $pdo->prepare('SELECT COUNT(*) FROM users WHERE active = ?');
    $statement->execute([1]);

    return $statement->fetchColumn();
}

/**
 * @throws PDOException
 */
function chained(PDO $pdo): bool
{
    return $pdo->prepare('DELETE FROM sessions')->execute();
}

/**
 * @throws PDOException
 */
function query_string(PDO $pdo): string
{
    return $pdo->prepare('SELECT 1')->queryString;
}

/**
 * @throws PDOException
 */
function prepared(PDO $pdo): PDOStatement
{
    return $pdo->prepare('SELECT 1');
}

/**
 * @throws PDOException
 * @throws RuntimeException
 */
function checked(PDO $pdo): PDOStatement
{
    $statement = $pdo->prepare('SELECT 1');
    if ($statement === false) {
        throw new RuntimeException('Could not prepare the statement.');
    }

    return $statement;
}

function lenient_method(): int
{
    return lenient()->get();
}

function lenient_property(): int
{
    $box = lenient();

    return $box->value;
}

function strict_method(): int
{
    // @mago-expect analysis:invalid-method-access
    return strict()->get();
}

function strict_property(): ?int
{
    // @mago-expect analysis:invalid-property-access
    return strict()->value;
}
