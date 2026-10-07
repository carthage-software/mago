<?php

declare(strict_types=1);

function invalidProducerPayload(RdKafka\ProducerTopic $topic): void
{
    // @mago-expect analysis:invalid-argument
    $topic->produce(RD_KAFKA_PARTITION_UA, 0, new stdClass());
}

function invalidErrorCallback(RdKafka\Conf $conf): void
{
    // @mago-expect analysis:invalid-argument
    $conf->setErrorCb(static function (RdKafka|RdKafka\KafkaConsumer $client, string $error, string $reason): void {
        echo $client->poll(0), $error, $reason;
    });
}

function nonNullablePayload(RdKafka\Message $message): string
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $message->payload;
}

function wrongConstantType(): string
{
    // @mago-expect analysis:invalid-return-statement
    return RD_KAFKA_RESP_ERR_NO_ERROR;
}

/** @throws RdKafka\Exception */
function invalidPartitions(RdKafka\KafkaConsumer $consumer): void
{
    // @mago-expect analysis:invalid-argument
    $consumer->assign(['events']);
}

/** @throws RdKafka\Exception */
function missingMetadataArguments(RdKafka\Producer $producer): void
{
    // @mago-expect analysis:too-few-arguments
    $producer->getMetadata(true);
}
