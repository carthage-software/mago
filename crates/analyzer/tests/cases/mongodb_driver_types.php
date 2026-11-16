<?php

declare(strict_types=1);

use MongoDB\BSON\Binary;
use MongoDB\BSON\Document;
use MongoDB\BSON\ObjectId;
use MongoDB\BSON\PackedArray;
use MongoDB\BSON\UTCDateTime;
use MongoDB\BSON\VectorType;
use MongoDB\Driver\BulkWriteCommand;
use MongoDB\Driver\BulkWriteCommandResult;
use MongoDB\Driver\CursorInterface;
use MongoDB\Driver\Exception\ServerException;
use MongoDB\Driver\Manager;
use MongoDB\Driver\Monitoring\LogSubscriber;
use MongoDB\Driver\Query;
use MongoDB\Driver\ReadPreference;
use MongoDB\Driver\Server;
use MongoDB\Driver\ServerDescription;
use MongoDB\Driver\Session;
use MongoDB\Driver\WriteError;
use MongoDB\Driver\WriteResult;

/**
 * @return list<array|object>
 *
 * @throws MongoDB\Driver\Exception\Exception
 */
function find_all(Manager $manager): array
{
    $cursor = $manager->executeQuery('db.collection', new Query([]));
    $documents = [];
    foreach ($cursor as $index => $document) {
        $documents[$index] = $document;
    }

    return [...$documents, ...$cursor->toArray()];
}

/**
 * @param CursorInterface<array{name: string}> $cursor
 *
 * @return list<string>
 */
function names(CursorInterface $cursor): array
{
    $names = [];
    foreach ($cursor as $document) {
        $names[] = $document['name'];
    }

    return $names;
}

/**
 * @throws MongoDB\Driver\Exception\Exception
 */
function document_value(string $json): mixed
{
    $document = Document::fromJSON($json);
    // @mago-expect analysis:mixed-assignment
    foreach ($document as $key => $value) {
        echo strlen($key);
    }

    return $document->toPHP()->value ?? $document->get('value');
}

/**
 * @return list<mixed>
 *
 * @throws Exception
 */
function packed_values(PackedArray $array): array
{
    // @mago-expect analysis:mixed-assignment
    foreach ($array->getIterator() as $index => $value) {
        echo $index + 1;
    }

    return $array->toPHP();
}

/**
 * @return ServerDescription::TYPE_RS_PRIMARY|ServerDescription::TYPE_RS_SECONDARY|null
 */
function replica_member(Server $server): ?string
{
    $type = $server->getServerDescription()->getType();

    return match ($type) {
        ServerDescription::TYPE_RS_PRIMARY, ServerDescription::TYPE_RS_SECONDARY => $type,
        default => null,
    };
}

/**
 * @throws MongoDB\Driver\Exception\LogicException
 */
function in_transaction(Session $session): bool
{
    return $session->getTransactionState() === Session::TRANSACTION_IN_PROGRESS;
}

/**
 * @return list<int>
 *
 * @throws MongoDB\Driver\Exception\LogicException
 * @mago-expect analysis:too-wide-throw-type
 */
function failed_indexes(WriteResult $result): array
{
    return array_map(static fn(WriteError $error): int => $error->getIndex(), $result->getWriteErrors());
}

/**
 * @throws LogicException
 */
function server_error(ServerException $exception): never
{
    throw new LogicException('Server failure.', $exception->getCode(), $exception);
}

/**
 * @return non-empty-lowercase-string
 */
function object_id(ObjectId $id): string
{
    return (string) $id;
}

/**
 * @return array{'$date': array{'$numberLong': numeric-string}}
 */
function date_json(UTCDateTime $date): array
{
    return $date->jsonSerialize();
}

/**
 * @return int<0, 255>
 */
function binary_type(Binary $binary): int
{
    return $binary->getType();
}

/**
 * @throws MongoDB\Driver\Exception\InvalidArgumentException
 */
function vector(): Binary
{
    return Binary::fromVector([0.5, 1.5], VectorType::Float32);
}

/**
 * @throws MongoDB\Driver\Exception\Exception
 */
function bulk_write(Manager $manager): BulkWriteCommandResult
{
    $command = new BulkWriteCommand();
    $command->insertOne('db.collection', ['x' => 1]);

    return $manager->executeBulkWriteCommand($command);
}

final class Logger implements LogSubscriber
{
    public function log(int $level, string $domain, string $message): void
    {
        echo $level === LogSubscriber::LEVEL_DEBUG ? $message : $domain;
    }
}

/**
 * @throws MongoDB\Driver\Exception\InvalidArgumentException
 */
function secondary(): ReadPreference
{
    $preference = new ReadPreference(ReadPreference::SECONDARY);
    // @mago-expect analysis:impossible-condition,redundant-comparison
    if ($preference->getMaxStalenessSeconds() === 10) {
        echo 'unreachable';
    }

    return $preference;
}

/**
 * @throws MongoDB\Driver\Exception\LogicException
 */
function document_is_immutable(Document $document): int
{
    $document->offsetSet('x', 1);
}
