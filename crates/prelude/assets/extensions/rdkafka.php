<?php

// SPDX-License-Identifier: Apache-2.0
// Copyright 2010-2023 JetBrains s.r.o.
// Modified and consolidated from JetBrains/phpstorm-stubs:
// https://github.com/JetBrains/phpstorm-stubs/tree/e4f5f6c3de39f3bab3e9f3fca4b8cdb8b061e681/rdkafka

namespace {
    /** @var int */
    const RD_KAFKA_RESP_ERR__BEGIN = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__BAD_MSG = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__BAD_COMPRESSION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__DESTROY = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__FAIL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__TRANSPORT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__CRIT_SYS_RESOURCE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__RESOLVE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__MSG_TIMED_OUT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__PARTITION_EOF = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNKNOWN_PARTITION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__FS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNKNOWN_TOPIC = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__ALL_BROKERS_DOWN = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__INVALID_ARG = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__TIMED_OUT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__QUEUE_FULL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__ISR_INSUFF = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NODE_UPDATE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__SSL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__WAIT_COORD = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNKNOWN_GROUP = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__PREV_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__EXISTING_SUBSCRIPTION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__ASSIGN_PARTITIONS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__REVOKE_PARTITIONS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__CONFLICT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__STATE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNKNOWN_PROTOCOL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NOT_IMPLEMENTED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__AUTHENTICATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NO_OFFSET = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__OUTDATED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__TIMED_OUT_QUEUE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNSUPPORTED_FEATURE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__WAIT_CACHE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__INTR = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__KEY_SERIALIZATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__VALUE_SERIALIZATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__KEY_DESERIALIZATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__VALUE_DESERIALIZATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__PARTIAL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__END = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNKNOWN = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NO_ERROR = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_OFFSET_OUT_OF_RANGE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_MSG = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNKNOWN_TOPIC_OR_PART = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_MSG_SIZE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_LEADER_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_LEADER_FOR_PARTITION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_BROKER_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_REPLICA_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_MSG_SIZE_TOO_LARGE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_STALE_CTRL_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_OFFSET_METADATA_TOO_LARGE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NETWORK_EXCEPTION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_LOAD_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_COORDINATOR_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_COORDINATOR_FOR_GROUP = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TOPIC_EXCEPTION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_RECORD_LIST_TOO_LARGE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_ENOUGH_REPLICAS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_ENOUGH_REPLICAS_AFTER_APPEND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_REQUIRED_ACKS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_ILLEGAL_GENERATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INCONSISTENT_GROUP_PROTOCOL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_GROUP_ID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNKNOWN_MEMBER_ID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_SESSION_TIMEOUT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_REBALANCE_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_COMMIT_OFFSET_SIZE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TOPIC_AUTHORIZATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_AUTHORIZATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_CLUSTER_AUTHORIZATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_TIMESTAMP = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNSUPPORTED_SASL_MECHANISM = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_ILLEGAL_SASL_STATE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNSUPPORTED_VERSION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TOPIC_ALREADY_EXISTS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_PARTITIONS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_REPLICATION_FACTOR = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_REPLICA_ASSIGNMENT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_CONFIG = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_CONTROLLER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_REQUEST = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNSUPPORTED_FOR_MESSAGE_FORMAT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_POLICY_VIOLATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_OUT_OF_ORDER_SEQUENCE_NUMBER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DUPLICATE_SEQUENCE_NUMBER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_PRODUCER_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_TXN_STATE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_PRODUCER_ID_MAPPING = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_TRANSACTION_TIMEOUT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_CONCURRENT_TRANSACTIONS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TRANSACTION_COORDINATOR_FENCED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TRANSACTIONAL_ID_AUTHORIZATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_SECURITY_DISABLED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_OPERATION_NOT_ATTEMPTED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_COORDINATOR_LOAD_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_COORDINATOR_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NOT_COORDINATOR = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_KAFKA_STORAGE_ERROR = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_LOG_DIR_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_SASL_AUTHENTICATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNKNOWN_PRODUCER_ID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_REASSIGNMENT_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_AUTH_DISABLED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_OWNER_MISMATCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_REQUEST_NOT_ALLOWED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_AUTHORIZATION_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DELEGATION_TOKEN_EXPIRED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_PRINCIPAL_TYPE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NON_EMPTY_GROUP = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_ID_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_FETCH_SESSION_ID_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_FETCH_SESSION_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_LISTENER_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_TOPIC_DELETION_DISABLED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_FENCED_LEADER_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNKNOWN_LEADER_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNSUPPORTED_COMPRESSION_TYPE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_STALE_BROKER_EPOCH = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_OFFSET_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_MEMBER_ID_REQUIRED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_PREFERRED_LEADER_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_MAX_SIZE_REACHED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_FENCED_INSTANCE_ID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_ELIGIBLE_LEADERS_NOT_AVAILABLE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_ELECTION_NOT_NEEDED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_NO_REASSIGNMENT_IN_PROGRESS = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_GROUP_SUBSCRIBED_TO_TOPIC = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_RECORD = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNSTABLE_OFFSET_COMMIT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__READ_ONLY = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NOENT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNDERFLOW = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__INVALID_TYPE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__RETRY = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__PURGE_QUEUE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__PURGE_INFLIGHT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__FATAL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__INCONSISTENT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__GAPLESS_GUARANTEE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__MAX_POLL_EXCEEDED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__UNKNOWN_BROKER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NOT_CONFIGURED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__FENCED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__APPLICATION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__ASSIGNMENT_LOST = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__NOOP = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR__AUTO_OFFSET_RESET = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_THROTTLING_QUOTA_EXCEEDED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_PRODUCER_FENCED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_RESOURCE_NOT_FOUND = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_DUPLICATE_RESOURCE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_UNACCEPTABLE_CREDENTIAL = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INCONSISTENT_VOTER_SET = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_INVALID_UPDATE_VERSION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_FEATURE_UPDATE_FAILED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_RESP_ERR_PRINCIPAL_DESERIALIZATION_FAILURE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_CONSUMER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_OFFSET_BEGINNING = UNKNOWN;

    /** @var int */
    const RD_KAFKA_OFFSET_END = UNKNOWN;

    /** @var int */
    const RD_KAFKA_OFFSET_STORED = UNKNOWN;

    /** @var int */
    const RD_KAFKA_OFFSET_INVALID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_PARTITION_UA = UNKNOWN;

    /** @var int */
    const RD_KAFKA_PRODUCER = UNKNOWN;

    /** @var int */
    const RD_KAFKA_VERSION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_BUILD_VERSION = UNKNOWN;

    /** @var int */
    const RD_KAFKA_CONF_UNKNOWN = UNKNOWN;

    /** @var int */
    const RD_KAFKA_CONF_INVALID = UNKNOWN;

    /** @var int */
    const RD_KAFKA_CONF_OK = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_PARTITIONER_RANDOM = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_PARTITIONER_CONSISTENT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_PARTITIONER_CONSISTENT_RANDOM = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_PARTITIONER_MURMUR2 = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_PARTITIONER_MURMUR2_RANDOM = UNKNOWN;

    /** @var int */
    const RD_KAFKA_MSG_F_BLOCK = UNKNOWN;

    /** @var int */
    const RD_KAFKA_LOG_PRINT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_LOG_SYSLOG = UNKNOWN;

    /** @var int */
    const RD_KAFKA_LOG_SYSLOG_PRINT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_PURGE_F_QUEUE = UNKNOWN;

    /** @var int */
    const RD_KAFKA_PURGE_F_INFLIGHT = UNKNOWN;

    /** @var int */
    const RD_KAFKA_PURGE_F_NON_BLOCKING = UNKNOWN;

    /** @phpstan-return list<array{code: int, name: string|null, desc: string|null}> */
    function rd_kafka_get_err_descs() {}

    /** @return int */
    function rd_kafka_thread_cnt() {}

    /**
     * @param int $err
     *
     * @return string|null
     */
    function rd_kafka_err2str($err) {}

    function rd_kafka_err2name(int $err): ?string {}

    /**
     * @param int $errnox
     *
     * @return int
     * @deprecated
     */
    function rd_kafka_errno2err($errnox) {}

    /**
     * @return int
     * @deprecated
     */
    function rd_kafka_errno() {}

    /**
     * @param int $cnt
     *
     * @return int
     */
    function rd_kafka_offset_tail($cnt) {}

    abstract class RdKafka
    {
        /** @var callable|null */
        private $error_cb;

        /** @var callable|null */
        private $dr_cb;

        private function __construct() {}

        /**
         * @param string $broker_list
         *
         * @return int
         */
        public function addBrokers($broker_list) {}

        /**
         * @param bool $all_topics
         * @param RdKafka\Topic|null $only_topic
         * @param int $timeout_ms
         *
         * @throws RdKafka\Exception
         * @return RdKafka\Metadata
         */
        public function getMetadata($all_topics, $only_topic, $timeout_ms) {}

        public function getControllerId(int $timeout_ms): int {}

        /** @return int */
        public function getOutQLen() {}

        /**
         * @param string $topic_name
         * @param RdKafka\TopicConf|null $topic_conf
         *
         * @return RdKafka\Topic
         */
        public function newTopic($topic_name, $topic_conf = null) {}

        /**
         * @param int $timeout_ms
         *
         * @return int
         */
        public function poll($timeout_ms) {}

        /**
         * @param int $level
         *
         * @return void
         * @deprecated
         */
        public function setLogLevel($level) {}

        /**
         * @param RdKafka\TopicPartition[] $topic_partitions
         * @param int $timeout_ms
         *
         * @return list<RdKafka\TopicPartition>
         */
        public function offsetsForTimes($topic_partitions, $timeout_ms) {}

        /**
         * @param string $topic
         * @param int $partition
         * @param int $low
         * @param int $high
         * @param int $timeout_ms
         * @param-out int $low
         * @param-out int $high
         *
         * @return void
         */
        public function queryWatermarkOffsets($topic, $partition, &$low, &$high, $timeout_ms) {}

        /**
         * @param int $purge_flags
         *
         * @return int
         */
        public function purge($purge_flags) {}

        /**
         * @param int $timeout_ms
         *
         * @return int
         */
        public function flush($timeout_ms) {}

        /**
         * @param bool $all_topics
         * @param RdKafka\Topic|null $only_topic
         * @param int $timeout_ms
         *
         * @return RdKafka\Metadata
         * @deprecated
         */
        public function metadata($all_topics, $only_topic, $timeout_ms) {}

        /**
         * @param int $logger
         *
         * @return void
         * @deprecated
         */
        public function setLogger($logger) {}

        /**
         * @return int
         * @deprecated
         */
        public function outqLen() {}

        /**
         * @param RdKafka\TopicPartition[] $topic_partitions
         * @return list<RdKafka\TopicPartition>
         */
        public function pausePartitions(array $topic_partitions): array {}

        /**
         * @param RdKafka\TopicPartition[] $topic_partitions
         * @return list<RdKafka\TopicPartition>
         */
        public function resumePartitions(array $topic_partitions): array {}

        /**
         * @param int|float|string $lifetime_ms
         * @param array<string, string> $extensions
         *
         * @throws \InvalidArgumentException
         * @throws RdKafka\Exception
         */
        public function oauthbearerSetToken(
            string $token_value,
            int|float|string $lifetime_ms,
            string $principal_name,
            array $extensions = [],
        ): void {}

        public function oauthbearerSetTokenFailure(string $error): void {}
    }
}

namespace RdKafka {
    class Conf
    {
        public function __construct() {}

        /** @return array<string, string> */
        public function dump() {}

        /**
         * @param string $name
         * @param string $value
         *
         * @return void
         */
        public function set($name, $value) {}

        /**
         * @param TopicConf $topic_conf
         *
         * @return void
         */
        public function setDefaultTopicConf($topic_conf) {}

        /**
         * @param callable(\RdKafka, Message): void $callback
         *
         * @return void
         */
        public function setDrMsgCb($callback) {}

        /**
         * @param callable(\RdKafka|KafkaConsumer, int, string): void $callback
         *
         * @return void
         */
        public function setErrorCb($callback) {}

        /**
         * @param callable(KafkaConsumer, int, list<TopicPartition>): void $callback
         *
         * @return void
         */
        public function setRebalanceCb($callback) {}

        /**
         * @param callable(\RdKafka|KafkaConsumer, string, int): void $callback
         *
         * @return void
         */
        public function setStatsCb($callback) {}

        /**
         * @param callable(\RdKafka|KafkaConsumer, int, list<TopicPartition>): void $callback
         *
         * @return void
         */
        public function setOffsetCommitCb($callback) {}

        /**
         * @param callable(Message, \RdKafka): void $callback
         *
         * @return void
         */
        public function setConsumeCb($callback) {}

        /**
         * @param callable(\RdKafka|KafkaConsumer, int, string, string): void $callback
         *
         * @return void
         */
        public function setLogCb($callback) {}

        /** @param callable(\RdKafka|KafkaConsumer, string|null): void $callback */
        public function setOauthbearerTokenRefreshCb(callable $callback): void {}
    }

    class Consumer extends \RdKafka
    {
        /** @param Conf|null $conf */
        public function __construct($conf = null) {}

        /**
         * @param string $topic_name
         * @param TopicConf|null $topic_conf
         *
         * @return ConsumerTopic
         */
        public function newTopic($topic_name, ?TopicConf $topic_conf = null) {}

        /** @return Queue */
        public function newQueue() {}
    }

    class ConsumerTopic extends Topic
    {
        private function __construct() {}

        /**
         * @param int $partition
         * @param int $timeout_ms
         *
         * @return Message|null
         */
        public function consume($partition, $timeout_ms) {}

        /**
         * @param int $partition
         * @param int $offset
         * @param Queue $queue
         *
         * @return void
         */
        public function consumeQueueStart($partition, $offset, $queue) {}

        /**
         * @param int $partition
         * @param int $offset
         *
         * @return void
         */
        public function consumeStart($partition, $offset) {}

        /**
         * @param int $partition
         *
         * @return void
         */
        public function consumeStop($partition) {}

        /**
         * @param int $partition
         * @param int $offset
         *
         * @return void
         */
        public function offsetStore($partition, $offset) {}

        /**
         * @param int $partition
         * @param int $timeout_ms
         * @param callable(Message): void $callback
         *
         * @return int
         */
        public function consumeCallback($partition, $timeout_ms, $callback) {}

        /**
         * @param int $partition
         * @param int $timeout_ms
         * @param int $batch_size
         *
         * @return list<Message>
         */
        public function consumeBatch($partition, $timeout_ms, $batch_size) {}
    }

    class Exception extends \Exception {}

    class KafkaConsumer
    {
        /** @var callable|null */
        private $error_cb;

        /** @var callable|null */
        private $rebalance_cb;

        /** @var callable|null */
        private $dr_msg_cb;

        /** @param Conf $conf */
        public function __construct($conf) {}

        /**
         * @param TopicPartition[]|null $topic_partitions
         *
         * @throws Exception
         * @return void
         */
        public function assign($topic_partitions = null) {}

        /** @param TopicPartition[] $topic_partitions */
        public function incrementalAssign(array $topic_partitions): void {}

        /** @param TopicPartition[] $topic_partitions */
        public function incrementalUnassign(array $topic_partitions): void {}

        /**
         * @param Message|TopicPartition[]|null $message_or_offsets
         *
         * @throws Exception
         * @return void
         */
        public function commit($message_or_offsets = null) {}

        /**
         * @param Message|TopicPartition[]|null $message_or_offsets
         *
         * @throws Exception
         * @return void
         */
        public function commitAsync($message_or_offsets = null) {}

        /**
         * @param int $timeout_ms
         *
         * @throws Exception
         * @throws \InvalidArgumentException
         * @return Message
         */
        public function consume($timeout_ms) {}

        /**
         * @throws Exception
         * @return list<TopicPartition>
         */
        public function getAssignment() {}

        /**
         * @param bool $all_topics
         * @param Topic|null $only_topic
         * @param int $timeout_ms
         *
         * @throws Exception
         * @return Metadata
         */
        public function getMetadata($all_topics, $only_topic, $timeout_ms) {}

        /** @return list<string> */
        public function getSubscription() {}

        /**
         * @param string[] $topics
         *
         * @throws Exception
         * @return void
         */
        public function subscribe($topics) {}

        /**
         * @throws Exception
         * @return void
         */
        public function unsubscribe() {}

        /**
         * @param TopicPartition[] $topic_partitions
         * @param int $timeout_ms
         *
         * @return list<TopicPartition>
         */
        public function getCommittedOffsets($topic_partitions, $timeout_ms) {}

        /**
         * @param TopicPartition[] $topic_partitions
         * @param int $timeout_ms
         *
         * @return list<TopicPartition>
         */
        public function offsetsForTimes($topic_partitions, $timeout_ms) {}

        /**
         * @param string $topic
         * @param int $partition
         * @param int $low
         * @param int $high
         * @param int $timeout_ms
         * @param-out int $low
         * @param-out int $high
         *
         * @return void
         */
        public function queryWatermarkOffsets($topic, $partition, &$low, &$high, $timeout_ms) {}

        /**
         * @param TopicPartition[] $topic_partitions
         *
         * @return list<TopicPartition>
         */
        public function getOffsetPositions($topic_partitions) {}

        /**
         * @param string $topic_name
         * @param TopicConf|null $topic_conf
         *
         * @return KafkaConsumerTopic
         */
        public function newTopic($topic_name, $topic_conf = null) {}

        public function getControllerId(int $timeout_ms): int {}

        /**
         * @param TopicPartition[] $topic_partitions
         * @return list<TopicPartition>
         */
        public function pausePartitions(array $topic_partitions): array {}

        /**
         * @param TopicPartition[] $topic_partitions
         * @return list<TopicPartition>
         */
        public function resumePartitions(array $topic_partitions): array {}

        public function poll(int $timeout_ms): int {}

        /**
         * @param int|float|string $lifetime_ms
         * @param array<string, string> $extensions
         *
         * @throws \InvalidArgumentException
         * @throws Exception
         */
        public function oauthbearerSetToken(
            string $token_value,
            int|float|string $lifetime_ms,
            string $principal_name,
            array $extensions = [],
        ): void {}

        public function oauthbearerSetTokenFailure(string $error): void {}

        /** @return void */
        public function close() {}
    }

    class KafkaConsumerTopic extends Topic
    {
        private function __construct() {}

        /**
         * @param int $partition
         * @param int $offset
         *
         * @return void
         */
        public function offsetStore($partition, $offset) {}
    }

    class KafkaErrorException extends Exception
    {
        private string $error_string;
        private bool $isFatal;
        private bool $isRetriable;
        private bool $transactionRequiresAbort;

        /**
         * @param string $message
         * @param int $code
         * @param string $error_string
         * @param bool $isFatal
         * @param bool $isRetriable
         * @param bool $transactionRequiresAbort
         */
        public function __construct(
            $message,
            $code,
            $error_string,
            $isFatal,
            $isRetriable,
            $transactionRequiresAbort,
        ) {}

        /** @return string */
        public function getErrorString() {}

        /** @return bool */
        public function isFatal() {}

        /** @return bool */
        public function isRetriable() {}

        /** @return bool */
        public function transactionRequiresAbort() {}
    }

    class Message
    {
        /** @var int */
        public $err;

        /** @var string|null */
        public $topic_name;

        /** @var int|null */
        public $timestamp;

        /** @var int */
        public $partition;

        /** @var string|null */
        public $payload;

        /** @var int|null */
        public $len;

        /** @var string|null */
        public $key;

        /** @var int */
        public $offset;

        /** @var array<array-key, string> */
        public array $headers;

        /** @var string|null */
        public $opaque;

        /** @return string|null */
        public function errstr() {}
    }

    class Metadata
    {
        private function __construct() {}

        /** @return Metadata\Collection<Metadata\Broker> */
        public function getBrokers() {}

        /** @return Metadata\Collection<Metadata\Topic> */
        public function getTopics() {}

        /** @return int */
        public function getOrigBrokerId() {}

        /** @return string */
        public function getOrigBrokerName() {}
    }

    class Producer extends \RdKafka
    {
        /** @param Conf|null $conf */
        public function __construct($conf = null) {}

        /**
         * @param string $topic_name
         * @param TopicConf|null $topic_conf
         *
         * @return ProducerTopic
         */
        public function newTopic($topic_name, ?TopicConf $topic_conf = null) {}

        /**
         * @param int $timeout_ms
         *
         * @throws KafkaErrorException
         * @return void
         */
        public function initTransactions($timeout_ms) {}

        /**
         * @throws KafkaErrorException
         * @return void
         */
        public function beginTransaction() {}

        /**
         * @param int $timeout_ms
         *
         * @throws KafkaErrorException
         * @return void
         */
        public function commitTransaction($timeout_ms) {}

        /**
         * @param int $timeout_ms
         *
         * @throws KafkaErrorException
         * @return void
         */
        public function abortTransaction($timeout_ms) {}
    }

    class ProducerTopic extends Topic
    {
        private function __construct() {}

        /**
         * @param int $partition
         * @param int $msgflags
         * @param string|null $payload
         * @param string|null $key
         * @param string|null $msg_opaque
         *
         * @return void
         */
        public function produce($partition, $msgflags, $payload = null, $key = null, $msg_opaque = null) {}

        /**
         * @param int $partition
         * @param int $msgflags
         * @param string|null $payload
         * @param string|null $key
         * @param array<string, string|int|float|bool|\Stringable|null>|null $headers
         * @param int|null $timestamp_ms
         * @param string|null $msg_opaque
         *
         * @return void
         */
        public function producev(
            $partition,
            $msgflags,
            $payload = null,
            $key = null,
            $headers = null,
            $timestamp_ms = null,
            $msg_opaque = null,
        ) {}
    }

    class Queue
    {
        private function __construct() {}

        /**
         * @param int $timeout_ms
         *
         * @return Message|null
         */
        public function consume($timeout_ms) {}
    }

    abstract class Topic
    {
        /** @return string */
        public function getName() {}
    }

    class TopicConf
    {
        public function __construct() {}

        /** @return array<string, string> */
        public function dump() {}

        /**
         * @param string $name
         * @param string $value
         *
         * @return void
         */
        public function set($name, $value) {}

        /**
         * @param int $partitioner
         *
         * @return void
         */
        public function setPartitioner($partitioner) {}
    }

    class TopicPartition
    {
        /**
         * @param string $topic
         * @param int $partition
         * @param int $offset
         */
        public function __construct($topic, $partition, int $offset = 0) {}

        /** @return int */
        public function getOffset() {}

        /** @return int */
        public function getPartition() {}

        /** @return string|null */
        public function getTopic() {}

        /**
         * @param int $offset
         *
         * @return self
         */
        public function setOffset($offset) {}

        /**
         * @param int $partition
         *
         * @return self
         */
        public function setPartition($partition) {}

        /**
         * @param string $topic_name
         *
         * @return self
         */
        public function setTopic($topic_name) {}

        public function getErr(): ?int {}
    }
}

namespace RdKafka\Metadata {
    class Broker
    {
        private function __construct() {}

        /** @return int */
        public function getId() {}

        /** @return string */
        public function getHost() {}

        /** @return int */
        public function getPort() {}
    }

    /**
     * @template-covariant T
     * @implements \Iterator<int, T>
     */
    class Collection implements \Iterator, \Countable
    {
        private function __construct() {}

        /**
         * @throws \RdKafka\Exception
         * @return T
         */
        public function current(): mixed {}

        public function next(): void {}

        /** @throws \RdKafka\Exception */
        public function key(): int {}

        public function valid(): bool {}

        public function rewind(): void {}

        public function count(): int {}
    }

    class Partition
    {
        private function __construct() {}

        /** @return int */
        public function getId() {}

        /** @return int */
        public function getErr() {}

        /** @return int */
        public function getLeader() {}

        /** @return Collection<int> */
        public function getReplicas() {}

        /** @return Collection<int> */
        public function getIsrs() {}
    }

    class Topic
    {
        private function __construct() {}

        /** @return string */
        public function getTopic() {}

        /** @return Collection<Partition> */
        public function getPartitions() {}

        /** @return int */
        public function getErr() {}
    }
}
