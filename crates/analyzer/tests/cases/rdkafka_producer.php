<?php

declare(strict_types=1);

function publishRecord(string $payload, Stringable $headerValue): int
{
    $conf = new RdKafka\Conf();
    $conf->set('bootstrap.servers', 'localhost:9092');
    $conf->setDrMsgCb(static function (RdKafka $client, RdKafka\Message $message): void {
        echo $client->getOutQLen(), $message->err, $message->payload ?? '';
    });
    $conf->setErrorCb(static function (RdKafka|RdKafka\KafkaConsumer $client, int $error, string $reason): void {
        echo $client->poll(0), $error, $reason;
    });

    $producer = new RdKafka\Producer($conf);
    $topicConf = new RdKafka\TopicConf();
    $topicConf->setPartitioner(RD_KAFKA_MSG_PARTITIONER_MURMUR2_RANDOM);
    $topic = $producer->newTopic('events', $topicConf);
    echo $topic->getName();
    $topic->produce(RD_KAFKA_PARTITION_UA, RD_KAFKA_MSG_F_BLOCK, $payload, null);
    $topic->producev(RD_KAFKA_PARTITION_UA, 0, null, 'deleted-key', [
        'trace' => null,
        'attempt' => 1,
        'enabled' => true,
        'ratio' => 1.5,
        'identifier' => $headerValue,
    ]);

    return $producer->poll(0) + $producer->flush(1000);
}

/** @throws RdKafka\KafkaErrorException */
function publishTransaction(RdKafka\Producer $producer): void
{
    $producer->initTransactions(1000);
    $producer->beginTransaction();
    $producer->newTopic('events')->produce(RD_KAFKA_PARTITION_UA, 0, 'transaction');
    $producer->commitTransaction(1000);
}
