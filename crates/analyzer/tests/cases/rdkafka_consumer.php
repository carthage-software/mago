<?php

declare(strict_types=1);

/** @throws RdKafka\Exception */
function createConsumer(): RdKafka\KafkaConsumer
{
    $conf = new RdKafka\Conf();
    $conf->set('group.id', 'events');
    $conf->setRebalanceCb(
        /**
         * @param list<RdKafka\TopicPartition> $partitions
         * @throws RdKafka\Exception
         */
        static function (RdKafka\KafkaConsumer $consumer, int $error, array $partitions): void {
            if ($error === RD_KAFKA_RESP_ERR__ASSIGN_PARTITIONS) {
                $consumer->assign($partitions);
            } else {
                $consumer->assign(null);
            }
        },
    );
    $conf->setOffsetCommitCb(
        /** @param list<RdKafka\TopicPartition> $partitions */
        static function (RdKafka|RdKafka\KafkaConsumer $consumer, int $error, array $partitions): void {
            echo $consumer->poll(0), $error;
            foreach ($partitions as $partition) {
                echo $partition->getOffset();
            }
        },
    );
    $consumer = new RdKafka\KafkaConsumer($conf);
    $consumer->subscribe(['events']);

    return $consumer;
}

/**
 * @throws RdKafka\Exception
 * @throws InvalidArgumentException
 */
function nextPayload(RdKafka\KafkaConsumer $consumer): ?string
{
    $message = $consumer->consume(1000);
    if ($message->err !== RD_KAFKA_RESP_ERR_NO_ERROR) {
        echo $message->errstr() ?? '';
        return null;
    }

    $consumer->commit($message);
    echo $message->key ?? '', $message->topic_name ?? '', $message->offset;
    foreach ($message->headers as $name => $value) {
        echo $name, $value;
    }

    return $message->payload;
}

/**
 * @throws RdKafka\Exception
 * @return list<RdKafka\TopicPartition>
 */
function committedPartitions(RdKafka\KafkaConsumer $consumer): array
{
    $partitions = $consumer->getAssignment();
    $consumer->commitAsync($partitions);

    return $consumer->getCommittedOffsets($partitions, 1000);
}

function nextLegacyMessage(RdKafka\Consumer $consumer): ?RdKafka\Message
{
    $topic = $consumer->newTopic('events');
    $topic->consumeStart(0, RD_KAFKA_OFFSET_BEGINNING);
    $message = $topic->consume(0, 1000);
    $topic->consumeStop(0);

    return $message;
}

function nextQueuedPayload(RdKafka\Consumer $consumer): ?string
{
    return $consumer->newQueue()->consume(1000)?->payload;
}

/** @return list<RdKafka\Message> */
function nextBatch(RdKafka\ConsumerTopic $topic): array
{
    return $topic->consumeBatch(0, 1000, 10);
}

function setPartitionOffset(RdKafka\TopicPartition $partition): RdKafka\TopicPartition
{
    return $partition->setOffset(rd_kafka_offset_tail(10))->setPartition(0)->setTopic('events');
}
