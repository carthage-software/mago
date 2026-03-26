<?php

namespace {
    class Aerospike
    {
        public const OPT_READ_DEFAULT_POL = 'OPT_READ_DEFAULT_POL';

        public const OPT_WRITE_DEFAULT_POL = 'OPT_WRITE_DEFAULT_POL';

        public const OPT_REMOVE_DEFAULT_POL = 'OPT_REMOVE_DEFAULT_POL';

        public const OPT_BATCH_DEFAULT_POL = 'OPT_BATCH_DEFAULT_POL';

        public const OPT_OPERATE_DEFAULT_POL = 'OPT_OPERATE_DEFAULT_POL';

        public const OPT_QUERY_DEFAULT_POL = 'OPT_QUERY_DEFAULT_POL';

        public const OPT_SCAN_DEFAULT_POL = 'OPT_SCAN_DEFAULT_POL';

        public const OPT_APPLY_DEFAULT_POL = 'OPT_APPLY_DEFAULT_POL';

        public const OPT_TLS_CONFIG = 'OPT_TLS_CONFIG';

        public const OPT_TLS_ENABLE = 'OPT_TLS_ENABLE';

        public const OPT_OPT_TLS_CAFILE = 'OPT_OPT_TLS_CAFILE';

        public const OPT_TLS_CAPATH = 'OPT_TLS_CAPATH';

        public const OPT_TLS_PROTOCOLS = 'OPT_TLS_PROTOCOLS';

        public const OPT_TLS_CIPHER_SUITE = 'OPT_TLS_CIPHER_SUITE';

        public const OPT_TLS_CRL_CHECK = 'OPT_TLS_CRL_CHECK';

        public const OPT_TLS_CRL_CHECK_ALL = 'OPT_TLS_CRL_CHECK_ALL';

        public const OPT_TLS_CERT_BLACKLIST = 'OPT_TLS_CERT_BLACKLIST';

        public const OPT_TLS_LOG_SESSION_INFO = 'OPT_TLS_LOG_SESSION_INFO';

        public const OPT_TLS_KEYFILE = 'OPT_TLS_KEYFILE';

        public const OPT_TLS_CERTFILE = 'OPT_TLS_CERTFILE';

        public const OPT_CONNECT_TIMEOUT = 'OPT_CONNECT_TIMEOUT';

        public const OPT_READ_TIMEOUT = 'OPT_READ_TIMEOUT';

        public const OPT_WRITE_TIMEOUT = 'OPT_WRITE_TIMEOUT';

        public const OPT_TTL = 'OPT_TTL';

        public const OPT_POLICY_KEY = 'OPT_POLICY_KEY';

        public const POLICY_KEY_DIGEST = 0;

        public const POLICY_KEY_SEND = 1;

        public const OPT_POLICY_EXISTS = 'OPT_POLICY_EXISTS';

        public const POLICY_EXISTS_IGNORE = 0;

        public const POLICY_EXISTS_CREATE = 1;

        public const POLICY_EXISTS_UPDATE = 2;

        public const POLICY_EXISTS_REPLACE = 3;

        public const POLICY_EXISTS_CREATE_OR_REPLACE = 4;

        public const OPT_POLICY_GEN = 'OPT_POLICY_GEN';

        public const POLICY_GEN_IGNORE = 0;

        public const POLICY_GEN_EQ = 1;

        public const POLICY_GEN_GT = 2;

        public const OPT_SERIALIZER = 'OPT_SERIALIZER';

        public const SERIALIZER_NONE = 0;

        public const SERIALIZER_PHP = 1;

        public const SERIALIZER_USER = 2;

        public const OPT_POLICY_COMMIT_LEVEL = 'OPT_POLICY_COMMIT_LEVEL';

        public const POLICY_COMMIT_LEVEL_ALL = 0;

        public const POLICY_COMMIT_LEVEL_MASTER = 1;

        public const OPT_POLICY_REPLICA = 'OPT_POLICY_REPLICA';

        public const POLICY_REPLICA_MASTER = 0;

        public const POLICY_REPLICA_ANY = 1;

        public const POLICY_REPLICA_SEQUENCE = 2;

        public const POLICY_REPLICA_PREFER_RACK = 3;

        public const OPT_POLICY_READ_MODE_AP = 'OPT_POLICY_READ_MODE_AP';

        public const POLICY_READ_MODE_AP_ONE = 0;

        public const AS_POLICY_READ_MODE_AP_ALL = 1;

        public const OPT_POLICY_READ_MODE_SC = 'OPT_POLICY_READ_MODE_SC';

        public const POLICY_READ_MODE_SC_SESSION = 0;

        public const POLICY_READ_MODE_SC_LINEARIZE = 1;

        public const POLICY_READ_MODE_SC_ALLOW_REPLICA = 2;

        public const POLICY_READ_MODE_SC_ALLOW_UNAVAILABLE = 3;

        public const OPT_DESERIALIZE = 'deserialize';

        public const OPT_SLEEP_BETWEEN_RETRIES = 'sleep_between_retries';

        public const OPT_MAX_RETRIES = 'OPT_MAX_RETRIES';

        public const OPT_TOTAL_TIMEOUT = 'OPT_TOTAL_TIMEOUT';

        public const OPT_SOCKET_TIMEOUT = 'OPT_SOCKET_TIMEOUT';

        public const OPT_BATCH_CONCURRENT = 'OPT_BATCH_CONCURRENT';

        public const OPT_ALLOW_INLINE = 'OPT_ALLOW_INLINE';

        public const OPT_SEND_SET_NAME = 'OPT_SEND_SET_NAME';

        public const OPT_FAIL_ON_CLUSTER_CHANGE = 'OPT_FAIL_ON_CLUSTER_CHANGE';

        public const OPT_SCAN_PRIORITY = 'OPT_SCAN_PRIORITY';

        public const SCAN_PRIORITY_AUTO = 'SCAN_PRIORITY_AUTO';

        public const SCAN_PRIORITY_LOW = 'SCAN_PRIORITY_LOW';

        public const SCAN_PRIORITY_MEDIUM = 'SCAN_PRIORITY_MEDIUM';

        public const SCAN_PRIORITY_HIGH = 'SCAN_PRIORITY_HIGH';

        public const OPT_SCAN_NOBINS = 'OPT_SCAN_NOBINS';

        public const OPT_SCAN_PERCENTAGE = 'OPT_SCAN_PERCENTAGE';

        public const OPT_SCAN_CONCURRENTLY = 'OPT_SCAN_CONCURRENTLY';

        public const OPT_QUERY_NOBINS = 'OPT_QUERY_NOBINS';

        public const USE_BATCH_DIRECT = 'USE_BATCH_DIRECT';

        public const OPT_POLICY_DURABLE_DELETE = 'OPT_POLICY_DURABLE_DELETE';

        public const OPT_MAP_ORDER = 'OPT_MAP_ORDER';

        public const AS_MAP_UNORDERED = 'AS_MAP_UNORDERED';

        public const AS_MAP_KEY_ORDERED = 'AS_MAP_KEY_ORDERED';

        public const AS_MAP_KEY_VALUE_ORDERED = 'AS_MAP_KEY_VALUE_ORDERED';

        public const OPT_MAP_WRITE_MODE = 'OPT_MAP_WRITE_MODE';

        public const AS_MAP_UPDATE = 'AS_MAP_UPDATE';

        public const AS_MAP_UPDATE_ONLY = 'AS_MAP_UPDATE_ONLY';

        public const AS_MAP_CREATE_ONLY = 'AS_MAP_CREATE_ONLY';

        public const OPT_MAP_WRITE_FLAGS = 'OPT_MAP_WRITE_FLAGS';

        public const AS_MAP_WRITE_DEFAULT = 'AS_MAP_WRITE_DEFAULT';

        public const AS_MAP_WRITE_CREATE_ONLY = 'AS_MAP_WRITE_CREATE_ONLY';

        public const AS_MAP_WRITE_UPDATE_ONLY = 'AS_MAP_WRITE_UPDATE_ONLY';

        public const AS_MAP_WRITE_NO_FAIL = 'AS_MAP_WRITE_NO_FAIL';

        public const AS_MAP_WRITE_PARTIAL = 'AS_MAP_WRITE_PARTIAL';

        public const MAP_RETURN_NONE = 'AS_MAP_RETURN_NONE';

        public const MAP_RETURN_INDEX = 'AS_MAP_RETURN_INDEX';

        public const MAP_RETURN_REVERSE_INDEX = 'AS_MAP_RETURN_REVERSE_INDEX';

        public const MAP_RETURN_RANK = 'AS_MAP_RETURN_RANK';

        public const MAP_RETURN_REVERSE_RANK = 'AS_MAP_RETURN_REVERSE_RANK';

        public const MAP_RETURN_COUNT = 'AS_MAP_RETURN_COUNT';

        public const MAP_RETURN_KEY = 'AS_MAP_RETURN_KEY';

        public const MAP_RETURN_VALUE = 'AS_MAP_RETURN_VALUE';

        public const MAP_RETURN_KEY_VALUE = 'AS_MAP_RETURN_KEY_VALUE';

        public const LOG_LEVEL_OFF = 'LOG_LEVEL_OFF';

        public const LOG_LEVEL_ERROR = 'LOG_LEVEL_ERROR';

        public const LOG_LEVEL_WARN = 'LOG_LEVEL_WARN';

        public const LOG_LEVEL_INFO = 'LOG_LEVEL_INFO';

        public const LOG_LEVEL_DEBUG = 'LOG_LEVEL_DEBUG';

        public const LOG_LEVEL_TRACE = 'LOG_LEVEL_TRACE';

        public const OK = 'AEROSPIKE_OK';

        public const ERR_CONNECTION = 'AEROSPIKE_ERR_CONNECTION';

        public const ERR_TLS_ERROR = 'AEROSPIKE_ERR_TLS';

        public const ERR_INVALID_NODE = 'AEROSPIKE_ERR_INVALID_NODE';

        public const ERR_NO_MORE_CONNECTIONS = 'AEROSPIKE_ERR_NO_MORE_CONNECTIONS';

        public const ERR_ASYNC_CONNECTION = 'AEROSPIKE_ERR_ASYNC_CONNECTION';

        public const ERR_CLIENT_ABORT = 'AEROSPIKE_ERR_CLIENT_ABORT';

        public const ERR_INVALID_HOST = 'AEROSPIKE_ERR_INVALID_HOST';

        public const ERR_PARAM = 'AEROSPIKE_ERR_PARAM';

        public const ERR_CLIENT = 'AEROSPIKE_ERR_CLIENT';

        public const ERR_SERVER = 'AEROSPIKE_ERR_SERVER';

        public const ERR_RECORD_NOT_FOUND = 'AEROSPIKE_ERR_RECORD_NOT_FOUND';

        public const ERR_RECORD_GENERATION = 'AEROSPIKE_ERR_RECORD_GENERATION';

        public const ERR_REQUEST_INVALID = 'AEROSPIKE_ERR_REQUEST_INVALID';

        public const ERR_OP_NOT_APPLICABLE = 'AEROSPIKE_ERR_OP_NOT_APPLICABLE';

        public const ERR_RECORD_EXISTS = 'AEROSPIKE_ERR_RECORD_EXISTS';

        public const ERR_BIN_EXISTS = 'AEROSPIKE_ERR_BIN_EXISTS';

        public const ERR_CLUSTER_CHANGE = 'AEROSPIKE_ERR_CLUSTER_CHANGE';

        public const ERR_SERVER_FULL = 'AEROSPIKE_ERR_SERVER_FULL';

        public const ERR_TIMEOUT = 'AEROSPIKE_ERR_TIMEOUT';

        /**
         * @deprecated
         */
        public const ERR_ALWAYS_FORBIDDEN = 'AEROSPIKE_ERR_ALWAYS_FORBIDDEN';

        public const ERR_CLUSTER = 'AEROSPIKE_ERR_CLUSTER';

        public const ERR_BIN_INCOMPATIBLE_TYPE = 'AEROSPIKE_ERR_BIN_INCOMPATIBLE_TYPE';

        public const ERR_RECORD_TOO_BIG = 'AEROSPIKE_ERR_RECORD_TOO_BIG';

        public const ERR_RECORD_BUSY = 'AEROSPIKE_ERR_RECORD_BUSY';

        public const ERR_SCAN_ABORTED = 'AEROSPIKE_ERR_SCAN_ABORTED';

        public const ERR_UNSUPPORTED_FEATURE = 'AEROSPIKE_ERR_UNSUPPORTED_FEATURE';

        public const ERR_BIN_NOT_FOUND = 'AEROSPIKE_ERR_BIN_NOT_FOUND';

        public const ERR_DEVICE_OVERLOAD = 'AEROSPIKE_ERR_DEVICE_OVERLOAD';

        public const ERR_RECORD_KEY_MISMATCH = 'AEROSPIKE_ERR_RECORD_KEY_MISMATCH';

        public const ERR_NAMESPACE_NOT_FOUND = 'AEROSPIKE_ERR_NAMESPACE_NOT_FOUND';

        public const ERR_BIN_NAME = 'AEROSPIKE_ERR_BIN_NAME';

        public const ERR_FAIL_FORBIDDEN = 'AEROSPIKE_ERR_FORBIDDEN';

        public const ERR_FAIL_ELEMENT_NOT_FOUND = 'AEROSPIKE_ERR_FAIL_NOT_FOUND';

        public const ERR_FAIL_ELEMENT_EXISTS = 'AEROSPIKE_ERR_FAIL_ELEMENT_EXISTS';

        public const ERR_SECURITY_NOT_SUPPORTED = 'AEROSPIKE_ERR_SECURITY_NOT_SUPPORTED';

        public const ERR_SECURITY_NOT_ENABLED = 'AEROSPIKE_ERR_SECURITY_NOT_ENABLED';

        public const ERR_SECURITY_SCHEME_NOT_SUPPORTED = 'AEROSPIKE_ERR_SECURITY_SCHEME_NOT_SUPPORTED';

        public const ERR_INVALID_COMMAND = 'AEROSPIKE_ERR_INVALID_COMMAND';

        public const ERR_INVALID_FIELD = 'AEROSPIKE_ERR_INVALID_FIELD';

        public const ERR_ILLEGAL_STATE = 'AEROSPIKE_ERR_ILLEGAL_STATE';

        public const ERR_INVALID_USER = 'AEROSPIKE_ERR_INVALID_USER';

        public const ERR_USER_ALREADY_EXISTS = 'AEROSPIKE_ERR_USER_ALREADY_EXISTS';

        public const ERR_INVALID_PASSWORD = 'AEROSPIKE_ERR_INVALID_PASSWORD';

        public const ERR_EXPIRED_PASSWORD = 'AEROSPIKE_ERR_EXPIRED_PASSWORD';

        public const ERR_FORBIDDEN_PASSWORD = 'AEROSPIKE_ERR_FORBIDDEN_PASSWORD';

        public const ERR_INVALID_CREDENTIAL = 'AEROSPIKE_ERR_INVALID_CREDENTIAL';

        public const ERR_INVALID_ROLE = 'AEROSPIKE_ERR_INVALID_ROLE';

        public const ERR_INVALID_PRIVILEGE = 'AEROSPIKE_ERR_INVALID_PRIVILEGE';

        public const ERR_NOT_AUTHENTICATED = 'AEROSPIKE_ERR_NOT_AUTHENTICATED';

        public const ERR_ROLE_VIOLATION = 'AEROSPIKE_ERR_ROLE_VIOLATION';

        public const ERR_ROLE_ALREADY_EXISTS = 'AEROSPIKE_ERR_ROLE_ALREADY_EXISTS';

        public const ERR_UDF = 'AEROSPIKE_ERR_UDF';

        public const ERR_UDF_NOT_FOUND = 'AEROSPIKE_ERR_UDF_NOT_FOUND';

        public const ERR_LUA_FILE_NOT_FOUND = 'AEROSPIKE_ERR_LUA_FILE_NOT_FOUND';

        public const ERR_BATCH_DISABLED = 'AEROSPIKE_ERR_BATCH_DISABLED';

        public const ERR_BATCH_MAX_REQUESTS_EXCEEDED = 'AEROSPIKE_ERR_BATCH_MAX_REQUESTS_EXCEEDED';

        public const ERR_BATCH_QUEUES_FULL = 'AEROSPIKE_ERR_BATCH_QUEUES_FULL';

        public const ERR_GEO_INVALID_GEOJSON = 'AEROSPIKE_ERR_GEO_INVALID_GEOJSON';

        public const ERR_INDEX_FOUND = 'AEROSPIKE_ERR_INDEX_FOUND';

        public const ERR_INDEX_NOT_FOUND = 'AEROSPIKE_ERR_INDEX_NOT_FOUND';

        public const ERR_INDEX_OOM = 'AEROSPIKE_ERR_INDEX_OOM';

        public const ERR_INDEX_NOT_READABLE = 'AEROSPIKE_ERR_INDEX_NOT_READABLE';

        public const ERR_INDEX = 'AEROSPIKE_ERR_INDEX';

        public const ERR_INDEX_NAME_MAXLEN = 'AEROSPIKE_ERR_INDEX_NAME_MAXLEN';

        public const ERR_INDEX_MAXCOUNT = 'AEROSPIKE_ERR_INDEX_MAXCOUNT';

        public const ERR_QUERY_ABORTED = 'AEROSPIKE_ERR_QUERY_ABORTED';

        public const ERR_QUERY_QUEUE_FULL = 'AEROSPIKE_ERR_QUERY_QUEUE_FULL';

        public const ERR_QUERY_TIMEOUT = 'AEROSPIKE_ERR_QUERY_TIMEOUT';

        public const ERR_QUERY = 'AEROSPIKE_ERR_QUERY';

        public const OPERATOR_WRITE = 'OPERATOR_WRITE';

        public const OPERATOR_READ = 'OPERATOR_READ';

        public const OPERATOR_INCR = 'OPERATOR_INCR';

        public const OPERATOR_PREPEND = 'OPERATOR_PREPEND';

        public const OPERATOR_APPEND = 'OPERATOR_APPEND';

        public const OPERATOR_TOUCH = 'OPERATOR_TOUCH';

        public const OPERATOR_DELETE = 'OPERATOR_DELETE';

        public const OP_LIST_APPEND = 'OP_LIST_APPEND';

        public const OP_LIST_MERGE = 'OP_LIST_MERGE';

        public const OP_LIST_INSERT = 'OP_LIST_INSERT';

        public const OP_LIST_INSERT_ITEMS = 'OP_LIST_INSERT_ITEMS';

        public const OP_LIST_POP = 'OP_LIST_POP';

        public const OP_LIST_POP_RANGE = 'OP_LIST_POP_RANGE';

        public const OP_LIST_REMOVE = 'OP_LIST_REMOVE';

        public const OP_LIST_REMOVE_RANGE = 'OP_LIST_REMOVE_RANGE';

        public const OP_LIST_CLEAR = 'OP_LIST_CLEAR';

        public const OP_LIST_SET = 'OP_LIST_SET';

        public const OP_LIST_GET = 'OP_LIST_GET';

        public const OP_LIST_GET_RANGE = 'OP_LIST_GET_RANGE';

        public const OP_LIST_TRIM = 'OP_LIST_TRIM';

        public const OP_LIST_SIZE = 'OP_LIST_SIZE';

        public const OP_MAP_SIZE = 'OP_MAP_SIZE';

        public const OP_MAP_CLEAR = 'OP_MAP_CLEAR';

        public const OP_MAP_SET_POLICY = 'OP_MAP_SET_POLICY';

        public const OP_MAP_GET_BY_KEY = 'OP_MAP_GET_BY_KEY';

        public const OP_MAP_GET_BY_KEY_RANGE = 'OP_MAP_GET_BY_KEY_RANGE';

        public const OP_MAP_GET_BY_VALUE = 'OP_MAP_GET_BY_VALUE';

        public const OP_MAP_GET_BY_VALUE_RANGE = 'OP_MAP_GET_BY_VALUE_RANGE';

        public const OP_MAP_GET_BY_INDEX = 'OP_MAP_GET_BY_INDEX';

        public const OP_MAP_GET_BY_INDEX_RANGE = 'OP_MAP_GET_BY_INDEX_RANGE';

        public const OP_MAP_GET_BY_RANK = 'OP_MAP_GET_BY_RANK';

        public const OP_MAP_GET_BY_RANK_RANGE = 'OP_MAP_GET_BY_RANK_RANGE';

        public const OP_MAP_PUT = 'OP_MAP_PUT';

        public const OP_MAP_PUT_ITEMS = 'OP_MAP_PUT_ITEMS';

        public const OP_MAP_INCREMENT = 'OP_MAP_INCREMENT';

        public const OP_MAP_DECREMENT = 'OP_MAP_DECREMENT';

        public const OP_MAP_REMOVE_BY_KEY = 'OP_MAP_REMOVE_BY_KEY';

        public const OP_MAP_REMOVE_BY_KEY_LIST = 'OP_MAP_REMOVE_BY_KEY_LIST';

        public const OP_MAP_REMOVE_BY_KEY_RANGE = 'OP_MAP_REMOVE_BY_KEY_RANGE';

        public const OP_MAP_REMOVE_BY_VALUE = 'OP_MAP_REMOVE_BY_VALUE';

        public const OP_MAP_REMOVE_BY_VALUE_RANGE = 'OP_MAP_REMOVE_BY_VALUE_RANGE';

        public const OP_MAP_REMOVE_BY_VALUE_LIST = 'OP_MAP_REMOVE_BY_VALUE_LIST';

        public const OP_MAP_REMOVE_BY_INDEX = 'OP_MAP_REMOVE_BY_INDEX';

        public const OP_MAP_REMOVE_BY_INDEX_RANGE = 'OP_MAP_REMOVE_BY_INDEX_RANGE';

        public const OP_MAP_REMOVE_BY_RANK = 'OP_MAP_REMOVE_BY_RANK';

        public const OP_MAP_REMOVE_BY_RANK_RANGE = 'OP_MAP_REMOVE_BY_RANK_RANGE';

        public const OP_EQ = '=';

        public const OP_BETWEEN = 'BETWEEN';

        public const OP_CONTAINS = 'CONTAINS';

        public const OP_RANGE = 'RANGE';

        public const OP_GEOWITHINREGION = 'GEOWITHIN';

        public const OP_GEOCONTAINSPOINT = 'GEOCONTAINS';

        /**
         * @deprecated
         */
        public const SCAN_STATUS_UNDEF = 'SCAN_STATUS_UNDEF';

        /**
         * @deprecated
         */
        public const SCAN_STATUS_INPROGRESS = 'SCAN_STATUS_INPROGRESS';

        /**
         * @deprecated
         */
        public const SCAN_STATUS_ABORTED = 'SCAN_STATUS_ABORTED';

        /**
         * @deprecated
         */
        public const SCAN_STATUS_COMPLETED = 'SCAN_STATUS_COMPLETED';

        public const JOB_STATUS_UNDEF = 'JOB_STATUS_UNDEF';

        public const JOB_STATUS_INPROGRESS = 'JOB_STATUS_INPROGRESS';

        public const JOB_STATUS_COMPLETED = 'JOB_STATUS_COMPLETED';

        public const INDEX_TYPE_DEFAULT = 'INDEX_TYPE_DEFAULT';

        public const INDEX_TYPE_LIST = 'INDEX_TYPE_LIST';

        public const INDEX_TYPE_MAPKEYS = 'INDEX_TYPE_MAPKEYS';

        public const INDEX_TYPE_MAPVALUES = 'INDEX_TYPE_MAPVALUES';

        public const INDEX_STRING = 'INDEX_STRING';

        public const INDEX_NUMERIC = 'INDEX_NUMERIC';

        public const INDEX_GEO2DSPHERE = 'INDEX_GEO2DSPHERE';

        public const UDF_TYPE_LUA = 'UDF_TYPE_LUA';

        public const PRIV_READ = 'PRIV_READ';

        public const PRIV_READ_WRITE = 'PRIV_READ_WRITE';

        public const PRIV_READ_WRITE_UDF = 'PRIV_READ_WRITE_UDF';

        public const PRIV_USER_ADMIN = 'PRIV_USER_ADMIN';

        public const PRIV_DATA_ADMIN = 'PRIV_DATA_ADMIN';

        public const PRIV_SYS_ADMIN = 'PRIV_SYS_ADMIN';

        /**
         * @param array $config
         * @param bool $persistent_connection
         * @param array $options
         *
         * @no-named-arguments
         */
        public function __construct($config, $persistent_connection = true, array $options = []) {}

        /**
         * @return void
         */
        public function __destruct() {}

        /**
         * @return bool
         */
        public function isConnected() {}

        /**
         * @return void
         */
        public function close() {}

        /**
         * @return void
         */
        public function reconnect() {}

        /**
         * @return int|null null if not enabled
         */
        public function shmKey() {}

        /**
         * @return string
         */
        public function error() {}

        /**
         * @return int
         */
        public function errorno() {}

        /**
         * @param string $ns
         * @param string $set
         * @param int|string $pk
         * @param bool $is_digest
         *
         * @return array
         *
         * @no-named-arguments
         */
        public function initKey($ns, $set, $pk, $is_digest = false) {}

        /**
         * @param string $ns
         * @param string $set
         * @param int|string $pk
         *
         * @return string
         *
         * @no-named-arguments
         */
        public function getKeyDigest($ns, $set, $pk) {}

        /**
         * @param array $key
         * @param array $bins
         * @param int $ttl
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function put(array $key, array $bins, $ttl = 0, array $options = []) {}

        /**
         * @param array $key
         * @param array $record
         * @param null|array $select
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function get(array $key, &$record, $select = null, array $options = []) {}

        /**
         * @param array $key
         * @param array $metadata
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function exists(array $key, &$metadata, array $options = []) {}

        /**
         * @param array $key
         * @param int $ttl
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function touch(array $key, $ttl = 0, array $options = []) {}

        /**
         * @param array $key
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function remove(array $key, array $options = []) {}

        /**
         * @param array $key
         * @param array $bins
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function removeBin(array $key, array $bins, array $options = []) {}

        /**
         * @param string $ns
         * @param string $set
         * @param int    $nanos
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function truncate($ns, $set, $nanos, array $options = []) {}

        /**
         * @param array $key
         * @param string $bin
         * @param int|float $offset
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function increment(array $key, $bin, $offset, array $options = []) {}

        /**
         * @param array $key
         * @param string $bin
         * @param string $value
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function append(array $key, $bin, $value, array $options = []) {}

        /**
         * @param array $key
         * @param string $bin
         * @param string $value
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function prepend(array $key, $bin, $value, array $options = []) {}

        /**
         * @param array $key
         * @param array $operations
         * @param array $returned
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function operate(array $key, array $operations, &$returned, array $options = []) {}

        /**
         * @param array $key
         * @param array $operations
         * @param array $returned
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function operateOrdered(array $key, array $operations, &$returned, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $count
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listSize(array $key, $bin, &$count, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param mixed  $value
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listAppend(array $key, $bin, $value, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param array  $items
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listMerge(array $key, $bin, array $items, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param mixed  $value
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listInsert(array $key, $bin, $index, $value, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param array  $elements
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listInsertItems(array $key, $bin, $index, array $elements, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param mixed  $element
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listPop(array $key, $bin, $index, &$element, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param int    $count
         * @param array  $elements
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listPopRange(array $key, $bin, $index, $count, &$elements, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listRemove(array $key, $bin, $index, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param int    $count
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listRemoveRange(array $key, $bin, $index, $count, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param int    $count
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listTrim(array $key, $bin, $index, $count, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listClear(array $key, $bin, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param mixed  $value
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listSet(array $key, $bin, $index, $value, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param array  $element
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listGet(array $key, $bin, $index, array &$element, array $options = []) {}

        /**
         * @param array  $key
         * @param string $bin
         * @param int    $index
         * @param int    $count
         * @param array  $elements
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listGetRange(array $key, $bin, $index, $count, &$elements, array $options = []) {}

        /**
         * @param array $keys
         * @param array $records
         * @param array $select
         * @param array $options
         *
         * @return int
         */
        public function getMany(array $keys, &$records, array $select = [], array $options = []) {}

        /**
         * @param array $keys
         * @param array $metadata
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function existsMany(array $keys, array &$metadata, array $options = []) {}

        /**
         * @param string   $ns
         * @param string   $set
         * @param callable $record_cb
         * @param array    $select
         * @param array    $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function scan($ns, $set, callable $record_cb, array $select = [], array $options = []) {}

        /**
         * @param string    $ns
         * @param string    $set
         * @param array     $where
         * @param callable  $record_cb
         * @param array     $select
         * @param array     $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function query($ns, $set, array $where, callable $record_cb, array $select = [], array $options = []) {}

        /**
         * @param string     $bin
         * @param int|string $val
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateEquals($bin, $val) {}

        /**
         * @param string $bin
         * @param int    $min
         * @param int    $max
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateBetween($bin, $min, $max) {}

        /**
         * @param string     $bin
         * @param int        $index_type
         * @param int|string $val
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateContains($bin, $index_type, $val) {}

        /**
         * @param string $bin
         * @param int    $index_type
         * @param int    $min
         * @param int    $max
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateRange($bin, $index_type, $min, $max) {}

        /**
         * @param string $bin
         * @param string $point
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateGeoContainsGeoJSONPoint($bin, $point) {}

        /**
         * @param string $bin
         * @param float  $long
         * @param float  $lat
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateGeoContainsPoint($bin, $long, $lat) {}

        /**
         * @param string $bin
         * @param string $region
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateGeoWithinGeoJSONRegion($bin, $region) {}

        /**
         * @param string $bin
         * @param float  $long
         * @param float  $lat
         * @param float  $radiusMeter
         *
         * @return array
         *
         * @no-named-arguments
         */
        public static function predicateGeoWithinRadius($bin, $long, $lat, $radiusMeter) {}

        /**
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function jobInfo($job_id, $job_type, array &$info, array $options = []) {}

        /**
         * @param string $path
         * @param string $module
         * @param int    $language
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function register($path, $module, $language = Aerospike::UDF_TYPE_LUA, $options = []) {}

        /**
         * @param string $module
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function deregister($module, $options = []) {}

        /**
         * @param array $modules
         * @param int   $language
         * @param array $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function listRegistered(&$modules, $language = Aerospike::UDF_TYPE_LUA, $options = []) {}

        /**
         * @param string $module
         * @param string $code
         * @param string $language
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function getRegistered($module, &$code, $language = Aerospike::UDF_TYPE_LUA, $options = []) {}

        /**
         * @param array  $key
         * @param string $module
         * @param string $function
         * @param array  $args
         * @param mixed  $returned
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function apply(array $key, $module, $function, array $args = [], &$returned = null, $options = []) {}

        /**
         * @param string $ns
         * @param string $set
         * @param string $module
         * @param string $function
         * @param array  $args
         * @param int    $job_id
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function scanApply($ns, $set, $module, $function, array $args, &$job_id, array $options = []) {}

        /**
         * @param string $module
         * @param string $function
         * @param array  $args
         * @param int    $job_id
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function queryApply(
            $ns,
            $set,
            array $where,
            $module,
            $function,
            array $args,
            &$job_id,
            array $options = [],
        ) {}

        /**
         * @param string $module
         * @param string $function
         * @param array  $args
         * @param mixed  $returned
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function aggregate(
            $ns,
            $set,
            array $where,
            $module,
            $function,
            array $args,
            &$returned,
            array $options = [],
        ) {}

        /**
         * @param string $ns
         * @param string $set
         * @param string $bin
         * @param string $name
         * @param int    $indexType
         * @param int    $dataType
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function addIndex($ns, $set, $bin, $name, $indexType, $dataType, array $options = []) {}

        /**
         * @param string $ns
         * @param string $name
         * @param array  $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function dropIndex($ns, $name, array $options = []) {}

        /**
         * @param string        $request
         * @param string        $response
         * @param null|array    $host
         * @param array         $options
         *
         * @return int
         *
         * @no-named-arguments
         */
        public function info($request, &$response, $host = null, array $options = []) {}

        /**
         * @param string     $request
         * @param null|array $host
         * @param array      $options
         *
         * @return array
         *
         * @no-named-arguments
         */
        public function infoMany($request, $host = null, array $options = []) {}

        /**
         * @return array
         *
         * @no-named-arguments
         */
        public function getNodes() {}

        /**
         * @param int $log_level
         *
         * @no-named-arguments
         */
        public function setLogLevel($log_level) {}

        /**
         * @no-named-arguments
         */
        public function setLogHandler(callable $log_handler) {}

        /**
         * @no-named-arguments
         */
        public function setSerializer(callable $serialize_cb) {}

        /**
         * @no-named-arguments
         */
        public function setDeserializer(callable $unserialize_cb) {}
    }
}

namespace Aerospike {
    use Serializable;

    class Bytes implements Serializable
    {
        /**
         * @var string
         */
        public $s;

        /**
         * @param string $bin_str
         *
         * @no-named-arguments
         */
        public function __construct($bin_str) {}

        /**
         * @return string
         */
        public function serialize() {}

        /**
         * @param string $bin_str
         * @return string
         *
         * @no-named-arguments
         */
        public function unserialize($bin_str) {}

        /**
         * @return string
         */
        public function __toString() {}

        /**
         * @return string
         *
         * @no-named-arguments
         */
        public static function unwrap(Bytes $bytes_wrap) {}
    }
}
