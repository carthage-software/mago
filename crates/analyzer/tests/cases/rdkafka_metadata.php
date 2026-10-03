<?php

declare(strict_types=1);

/**
 * @throws RdKafka\Exception
 * @return list<string>
 */
function brokerAddresses(RdKafka\Producer $producer): array
{
    $addresses = [];
    $metadata = $producer->getMetadata(true, null, 1000);
    foreach ($metadata->getBrokers() as $broker) {
        $addresses[] = $broker->getHost() . ':' . $broker->getPort();
    }

    return $addresses;
}

/** @throws RdKafka\Exception */
function replicaCount(RdKafka\Metadata $metadata): int
{
    $count = 0;
    foreach ($metadata->getTopics() as $topic) {
        foreach ($topic->getPartitions() as $partition) {
            foreach ($partition->getReplicas() as $brokerId) {
                $count += $brokerId;
            }
        }
    }

    return $count;
}

/** @return list<int> */
function kafkaErrorCodes(): array
{
    $codes = [];
    foreach (rd_kafka_get_err_descs() as $description) {
        $codes[] = $description['code'];
        echo $description['name'] ?? '', $description['desc'] ?? '';
    }

    return $codes;
}

function watermarkSpan(RdKafka\KafkaConsumer $consumer): int
{
    $low = 0;
    $high = 0;
    $consumer->queryWatermarkOffsets('events', 0, $low, $high, 1000);

    return $high - $low;
}
