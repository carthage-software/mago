<?php

/**
 * Helper autocomplete for php redis cluster extension.
 * @see https://github.com/phpredis/phpredis/blob/develop/redis_cluster.stub.php
 *
 * Based on the phpredis-phpdoc by Max Kamashev (https://github.com/ukko/phpredis-phpdoc)
 *
 * @author Tommy Zheng <tommy@vlv.pw>
 * @link   https://github.com/zgb7mtr/phpredis_cluster_phpdoc
 */
class RedisCluster
{
    /**
     * Options
     */
    public const OPT_SLAVE_FAILOVER = 5;

    /**
     * Cluster options
     */
    public const FAILOVER_NONE = 0;
    public const FAILOVER_ERROR = 1;
    public const FAILOVER_DISTRIBUTE = 2;
    public const FAILOVER_DISTRIBUTE_SLAVES = 3;

    /**
     * Creates a Redis Cluster client
     *
     * @param string|null   $name
     * @param array|null    $seeds
     * @param int|float     $timeout
     * @param int|float     $read_timeout
     * @param bool          $persistent
     * @param mixed         $auth
     * @param array|null    $context
     * @throws RedisClusterException
     *
     * @example
     * <pre>
     * // Declaring a cluster with an array of seeds
     * $redisCluster = new RedisCluster(null,['127.0.0.1:6379']);
     *
     * // Loading a cluster configuration by name
     * // In order to load a named array, one must first define the seed nodes in redis.ini.
     * // The following lines would define the cluster 'mycluster', and be loaded automatically by phpredis.
     *
     * // # In redis.ini
     * // redis.clusters.seeds = "mycluster[]=localhost:7000&test[]=localhost:7001"
     * // redis.clusters.timeout = "mycluster=5"
     * // redis.clusters.read_timeout = "mycluster=10"
     * // redis.clusters.auth = "mycluster=password" OR ['user' => 'foo', 'pass' => 'bar] as example
     *
     * //Then, this cluster can be loaded by doing the following
     *
     * $redisClusterPro = new RedisCluster('mycluster');
     * $redisClusterDev = new RedisCluster('test');
     * </pre>
     */
    public function __construct(string|null $name, ?array $seeds = null, int|float $timeout = 0, int|float $read_timeout = 0, bool $persistent = false, #[\SensitiveParameter] mixed $auth = null, ?array $context = null) {}

    /**
     * Disconnects from the RedisCluster instance, except when pconnect is used.
     */
    public function close(): bool {}

    /**
     * Get the value related to the specified key
     *
     * @param   string $key
     *
     * @return  mixed If key didn't exist, FALSE is returned. Otherwise, the value related to this key is
     *                       returned.
     * @throws  RedisClusterException
     *
     * @link    https://redis.io/commands/get
     * @example
     * <pre>
     * $redisCluster->get('key');
     * </pre>
     */
    public function get(string $key): mixed {}

    /**
     * Set the string value in argument as value of the key.
     *
     * @since    If you're using Redis >= 2.6.12, you can pass extended options as explained in example
     *
     * @param   string    $key
     * @param   mixed    $value
     * @param   mixed $options If you pass an integer, phpredis will redirect to SETEX, and will try to use Redis
     *                             >= 2.6.12 extended options if you pass an array with valid values.
     *
     * @return  RedisCluster|string|bool TRUE if the command is successful.
     * @throws  RedisClusterException
     *
     * @link     https://redis.io/commands/set
     * @example
     * <pre>
     * // Simple key -> value set
     * $redisCluster->set('key', 'value');
     *
     * // Will redirect, and actually make an SETEX call
     * $redisCluster->set('key','value', 10);
     *
     * // Will set the key, if it doesn't exist, with a ttl of 10 seconds
     * $redisCluster->set('key', 'value', Array('nx', 'ex'=>10));
     *
     * // Will set a key, if it does exist, with a ttl of 1000 milliseconds
     * $redisCluster->set('key', 'value', Array('xx', 'px'=>1000));
     * </pre>
     */
    public function set(string $key, mixed $value, mixed $options = null): RedisCluster|string|bool {}

    /**
     * Returns the values of all specified keys.
     *
     * For every key that does not hold a string value or does not exist,
     * the special value false is returned. Because of this, the operation never fails.
     *
     * @param array $keys
     *
     * @return RedisCluster|array|false
     * @throws RedisClusterException
     *
     * @link https://redis.io/commands/mget
     * @example
     * <pre>
     * $redisCluster->del('x', 'y', 'z', 'h');    // remove x y z
     * $redisCluster->mset(array('x' => 'a', 'y' => 'b', 'z' => 'c'));
     * $redisCluster->hset('h', 'field', 'value');
     * var_dump($redisCluster->mget(array('x', 'y', 'z', 'h')));
     * // Output:
     * // array(3) {
     * // [0]=>
     * // string(1) "a"
     * // [1]=>
     * // string(1) "b"
     * // [2]=>
     * // string(1) "c"
     * // [3]=>
     * // bool(false)
     * // }
     * </pre>
     */
    public function mget(array $keys): RedisCluster|array|false {}

    /**
     * Sets multiple key-value pairs in one atomic command.
     * MSETNX only returns TRUE if all the keys were set (see SETNX).
     *
     * @param   array $key_values Pairs: array(key => value, ...)
     *
     * @return  RedisCluster|bool    TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/mset
     * @example
     * <pre>
     * $redisCluster->mset(array('key0' => 'value0', 'key1' => 'value1'));
     * var_dump($redisCluster->get('key0'));
     * var_dump($redisCluster->get('key1'));
     * // Output:
     * // string(6) "value0"
     * // string(6) "value1"
     * </pre>
     */
    public function mset(array $key_values): RedisCluster|bool {}

    /**
     * @param   array $key_values
     *
     * @return  RedisCluster|array|false 1 (if the keys were set) or 0 (no key was set)
     * @throws  RedisClusterException
     * @see     mset()
     *
     * @link    https://redis.io/commands/msetnx
     */
    public function msetnx(array $key_values): RedisCluster|array|false {}

    /**
     * Remove specified keys.
     *
     * @param string|array $key1 An array of keys, or an undefined number of parameters, each a key: key1 key2 key3
     *                            ... keyN
     * @param string ...$other_keys
     *
     * @return int Number of keys deleted.
     * @throws RedisClusterException
     * @link    https://redis.io/commands/del
     * @example
     * <pre>
     * $redisCluster->set('key1', 'val1');
     * $redisCluster->set('key2', 'val2');
     * $redisCluster->set('key3', 'val3');
     * $redisCluster->set('key4', 'val4');
     * $redisCluster->del('key1', 'key2');          // return 2
     * $redisCluster->del(array('key3', 'key4'));   // return 2
     * </pre>
     */
    public function del(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    /**
     * Set the string value in argument as value of the key, with a time to live.
     *
     * @param   string $key
     * @param   int    $expire
     * @param   mixed $value
     *
     * @return  RedisCluster|bool   TRUE if the command is successful.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/setex
     * @example
     * <pre>
     * $redisCluster->setex('key', 3600, 'value'); // sets key → value, with 1h TTL.
     * </pre>
     */
    public function setex(string $key, int $expire, mixed $value): RedisCluster|bool {}

    /**
     * PSETEX works exactly like SETEX with the sole difference that the expire time is specified in milliseconds
     * instead of seconds.
     *
     * @param   string $key
     * @param   int    $ttl
     * @param   string $value
     *
     * @return  RedisCluster|bool   TRUE if the command is successful.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/psetex
     * @example
     * <pre>
     * $redisCluster->psetex('key', 1000, 'value'); // sets key → value, with 1s TTL.
     * </pre>
     */
    public function psetex(string $key, int $timeout, string $value): RedisCluster|bool {}

    /**
     * Set the string value in argument as value of the key if the key doesn't already exist in the database.
     *
     * @param   string $key
     * @param   mixed $value
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/setnx
     * @example
     * <pre>
     * $redisCluster->setnx('key', 'value');   // return TRUE
     * $redisCluster->setnx('key', 'value');   // return FALSE
     * </pre>
     */
    public function setnx(string $key, mixed $value): RedisCluster|bool {}

    /**
     * Sets a value and returns the previous entry at that key.
     *
     * @param   string $key
     * @param   mixed $value
     *
     * @return  RedisCluster|string|bool  A string, the previous value located at this key.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/getset
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $exValue = $redisCluster->getset('x', 'lol');   // return '42', replaces x by 'lol'
     * $newValue = $redisCluster->get('x');            // return 'lol'
     * </pre>
     */
    public function getset(string $key, mixed $value): RedisCluster|string|bool {}

    /**
     * Verify if the specified key exists.
     *
     * @param   string $key
     *
     * @return  bool If the key exists, return TRUE, otherwise return FALSE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/exists
     * @example
     * <pre>
     * $redisCluster->set('key', 'value');
     * $redisCluster->exists('key');               //  TRUE
     * $redisCluster->exists('NonExistingKey');    // FALSE
     * </pre>
     */
    public function exists(mixed $key, mixed ...$other_keys): RedisCluster|int|bool {}

    /**
     * Returns the keys that match a certain pattern.
     *
     * @param   string $pattern pattern, using '*' as a wildcard.
     *
     * @return  RedisCluster|array|false   of STRING: The keys that match a certain pattern.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/keys
     * @example
     * <pre>
     * $allKeys = $redisCluster->keys('*');   // all keys will match this.
     * $keyWithUserPrefix = $redisCluster->keys('user*');
     * </pre>
     */
    public function keys(string $pattern): RedisCluster|array|false {}

    /**
     * Returns the type of data pointed by a given key.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false
     * @throws  RedisClusterException
     *
     * Depending on the type of the data pointed by the key,
     * this method will return the following value:
     * - string: RedisCluster::REDIS_STRING
     * - set:   RedisCluster::REDIS_SET
     * - list:  RedisCluster::REDIS_LIST
     * - zset:  RedisCluster::REDIS_ZSET
     * - hash:  RedisCluster::REDIS_HASH
     * - other: RedisCluster::REDIS_NOT_FOUND
     * @link    https://redis.io/commands/type
     * @example $redisCluster->type('key');
     */
    public function type(string $key): RedisCluster|int|false {}

    /**
     * Returns and removes the first element of the list.
     *
     * @param   string $key
     *
     * @return  RedisCluster|bool|string|array if command executed successfully BOOL FALSE in case of failure (empty list)
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lpop
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');
     * var_dump( $redisCluster->lRange('key1', 0, -1) );
     * // Output:
     * // array(3) {
     * //   [0]=> string(1) "A"
     * //   [1]=> string(1) "B"
     * //   [2]=> string(1) "C"
     * // }
     * $redisCluster->lPop('key1');
     * var_dump( $redisCluster->lRange('key1', 0, -1) );
     * // Output:
     * // array(2) {
     * //   [0]=> string(1) "B"
     * //   [1]=> string(1) "C"
     * // }
     * </pre>
     */
    public function lPop(string $key, int $count = 0): RedisCluster|bool|string|array {}

    /**
     * Returns and removes the last element of the list.
     *
     * @param   string $key
     *
     * @return  RedisCluster|bool|string|array if command executed successfully BOOL FALSE in case of failure (empty list)
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/rpop
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');
     * var_dump( $redisCluster->lRange('key1', 0, -1) );
     * // Output:
     * // array(3) {
     * //   [0]=> string(1) "A"
     * //   [1]=> string(1) "B"
     * //   [2]=> string(1) "C"
     * // }
     * $redisCluster->rPop('key1');
     * var_dump( $redisCluster->lRange('key1', 0, -1) );
     * // Output:
     * // array(2) {
     * //   [0]=> string(1) "A"
     * //   [1]=> string(1) "B"
     * // }
     * </pre>
     */
    public function rPop(string $key, int $count = 0): RedisCluster|bool|string|array {}

    /**
     * Set the list at index with the new value.
     *
     * @param string $key
     * @param int    $index
     * @param mixed $value
     *
     * @return RedisCluster|bool TRUE if the new value is setted. FALSE if the index is out of range, or data type identified by key
     * @throws RedisClusterException
     * is not a list.
     * @link    https://redis.io/commands/lset
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');  // key1 => [ 'A', 'B', 'C' ]
     * $redisCluster->lGet('key1', 0);     // 'A'
     * $redisCluster->lSet('key1', 0, 'X');
     * $redisCluster->lGet('key1', 0);     // 'X'
     * </pre>
     */
    public function lSet(string $key, int $index, mixed $value): RedisCluster|bool {}

    /**
     * Removes and returns a random element from the set value at Key.
     *
     * @param   string $key
     *
     * @return  RedisCluster|string|array|false  "popped" value, bool FALSE if set identified by key is empty or doesn't exist.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/spop
     * @example
     * <pre>
     * $redisCluster->sAdd('key1' , 'set1');
     * $redisCluster->sAdd('key1' , 'set2');
     * $redisCluster->sAdd('key1' , 'set3');
     * var_dump($redisCluster->sMembers('key1'));// 'key1' => {'set3', 'set1', 'set2'}
     * $redisCluster->sPop('key1');// 'set1'
     * var_dump($redisCluster->sMembers('key1'));// 'key1' => {'set3', 'set2'}
     * $redisCluster->sPop('key1');// 'set3',
     * var_dump($redisCluster->sMembers('key1'));// 'key1' => {'set2'}
     * </pre>
     */
    public function sPop(string $key, int $count = 0): RedisCluster|string|array|false {}

    /**
     * Adds the string values to the head (left) of the list. Creates the list if the key didn't exist.
     * If the key exists and is not a list, FALSE is returned.
     *
     * @param   string $key
     * @param   mixed $value String, value to push in key
     * @param   mixed ...$other_values
     *
     * @return  RedisCluster|int|boole    The new length of the list in case of success, FALSE in case of Failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lpush
     * @example
     * <pre>
     * $redisCluster->lPush('l', 'v1', 'v2', 'v3', 'v4')   // int(4)
     * var_dump( $redisCluster->lRange('l', 0, -1) );
     * //// Output:
     * // array(4) {
     * //   [0]=> string(2) "v4"
     * //   [1]=> string(2) "v3"
     * //   [2]=> string(2) "v2"
     * //   [3]=> string(2) "v1"
     * // }
     * </pre>
     */
    public function lPush(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|bool {}

    /**
     * Adds the string values to the tail (right) of the list. Creates the list if the key didn't exist.
     * If the key exists and is not a list, FALSE is returned.
     *
     * @param   string $key
     * @param   mixed ...$elements
     *
     * @return  RedisCluster|int|false     The new length of the list in case of success, FALSE in case of Failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/rpush
     * @example
     * <pre>
     * $redisCluster->rPush('r', 'v1', 'v2', 'v3', 'v4');    // int(4)
     * var_dump( $redisCluster->lRange('r', 0, -1) );
     * //// Output:
     * // array(4) {
     * //   [0]=> string(2) "v1"
     * //   [1]=> string(2) "v2"
     * //   [2]=> string(2) "v3"
     * //   [3]=> string(2) "v4"
     * // }
     * </pre>
     */
    public function rPush(string $key, mixed ...$elements): RedisCluster|int|false {}

    /**
     * BLPOP is a blocking list pop primitive.
     * It is the blocking version of LPOP because it blocks the connection when
     * there are no elements to pop from any of the given lists.
     * An element is popped from the head of the first list that is non-empty,
     * with the given keys being checked in the order that they are given.
     *
     * @param string|array $key Array containing the keys of the lists
     *                          Or STRING Key1 STRING Key2 STRING Key3 ... STRING Keyn
     * @param string|float|int $timeout_or_key Timeout or key
     *
     * @return  RedisCluster|array|null|false array('listName', 'element')
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/blpop
     * @example
     * <pre>
     * // Non blocking feature
     * $redisCluster->lPush('key1', 'A');
     * $redisCluster->del('key2');
     *
     * $redisCluster->blPop('key1', 'key2', 10); // array('key1', 'A')
     * // OR
     * $redisCluster->blPop(array('key1', 'key2'), 10); // array('key1', 'A')
     *
     * $redisCluster->brPop('key1', 'key2', 10); // array('key1', 'A')
     * // OR
     * $redisCluster->brPop(array('key1', 'key2'), 10); // array('key1', 'A')
     *
     * // Blocking feature
     *
     * // process 1
     * $redisCluster->del('key1');
     * $redisCluster->blPop('key1', 10);
     * // blocking for 10 seconds
     *
     * // process 2
     * $redisCluster->lPush('key1', 'A');
     *
     * // process 1
     * // array('key1', 'A') is returned
     * </pre>
     */
    public function blPop(string|array $key, string|float|int $timeout_or_key, mixed ...$extra_args): RedisCluster|array|null|false {}

    /**
     * BRPOP is a blocking list pop primitive.
     * It is the blocking version of RPOP because it blocks the connection when
     * there are no elements to pop from any of the given lists.
     * An element is popped from the tail of the first list that is non-empty,
     * with the given keys being checked in the order that they are given.
     * See the BLPOP documentation(https://redis.io/commands/blpop) for the exact semantics,
     * since BRPOP is identical to BLPOP with the only difference being that
     * it pops elements from the tail of a list instead of popping from the head.
     *
     * @param string|array $key Array containing the keys of the lists
     *                          Or STRING Key1 STRING Key2 STRING Key3 ... STRING Keyn
     * @param string|float|int $timeout_or_key Timeout or key
     *
     * @return  RedisCluster|array|null|false array('listName', 'element')
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/brpop
     * @example
     * <pre>
     * // Non blocking feature
     * $redisCluster->lPush('key1', 'A');
     * $redisCluster->del('key2');
     *
     * $redisCluster->blPop('key1', 'key2', 10); // array('key1', 'A')
     * // OR
     * $redisCluster->blPop(array('key1', 'key2'), 10); // array('key1', 'A')
     *
     * $redisCluster->brPop('key1', 'key2', 10); // array('key1', 'A')
     * // OR
     * $redisCluster->brPop(array('key1', 'key2'), 10); // array('key1', 'A')
     *
     * // Blocking feature
     *
     * // process 1
     * $redisCluster->del('key1');
     * $redisCluster->blPop('key1', 10);
     * // blocking for 10 seconds
     *
     * // process 2
     * $redisCluster->lPush('key1', 'A');
     *
     * // process 1
     * // array('key1', 'A') is returned
     * </pre>
     */
    public function brPop(string|array $key, string|float|int $timeout_or_key, mixed ...$extra_args): RedisCluster|array|null|false {}

    /**
     * Adds the string value to the tail (right) of the list if the ist exists. FALSE in case of Failure.
     *
     * @param   string $key
     * @param   string $value String, value to push in key
     *
     * @return  RedisCluster|bool|int     The new length of the list in case of success, FALSE in case of Failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/rpushx
     * @example
     * <pre>
     * $redisCluster->del('key1');
     * $redisCluster->rPushx('key1', 'A'); // returns 0
     * $redisCluster->rPush('key1', 'A'); // returns 1
     * $redisCluster->rPushx('key1', 'B'); // returns 2
     * $redisCluster->rPushx('key1', 'C'); // returns 3
     * // key1 now points to the following list: [ 'A', 'B', 'C' ]
     * </pre>
     */
    public function rPushx(string $key, string $value): RedisCluster|bool|int {}

    /**
     * Adds the string value to the head (left) of the list if the list exists.
     *
     * @param   string $key
     * @param   mixed $value Value to push in key
     *
     * @return  RedisCluster|int|bool     The new length of the list in case of success, FALSE in case of Failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lpushx
     * @example
     * <pre>
     * $redisCluster->del('key1');
     * $redisCluster->lPushx('key1', 'A');     // returns 0
     * $redisCluster->lPush('key1', 'A');      // returns 1
     * $redisCluster->lPushx('key1', 'B');     // returns 2
     * $redisCluster->lPushx('key1', 'C');     // returns 3
     * // key1 now points to the following list: [ 'C', 'B', 'A' ]
     * </pre>
     */
    public function lPushx(string $key, mixed $value): RedisCluster|int|bool {}

    /**
     * Insert value in the list before or after the pivot value. the parameter options
     * specify the position of the insert (before or after). If the list didn't exists,
     * or the pivot didn't exists, the value is not inserted.
     *
     * @param   string $key
     * @param   string $pos RedisCluster::BEFORE | RedisCluster::AFTER
     * @param   mixed $pivot
     * @param   mixed $value
     *
     * @return  RedisCluster|int|false     The number of the elements in the list, -1 if the pivot didn't exists.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/linsert
     * @example
     * <pre>
     * $redisCluster->del('key1');
     * $redisCluster->lInsert('key1', RedisCluster::AFTER, 'A', 'X');    // 0
     *
     * $redisCluster->lPush('key1', 'A');
     * $redisCluster->lPush('key1', 'B');
     * $redisCluster->lPush('key1', 'C');
     *
     * $redisCluster->lInsert('key1', RedisCluster::BEFORE, 'C', 'X');   // 4
     * $redisCluster->lRange('key1', 0, -1);                      // array('X', 'C', 'B', 'A')
     *
     * $redisCluster->lInsert('key1', RedisCluster::AFTER, 'C', 'Y');    // 5
     * $redisCluster->lRange('key1', 0, -1);                      // array('X', 'C', 'Y', 'B', 'A')
     *
     * $redisCluster->lInsert('key1', RedisCluster::AFTER, 'W', 'value'); // -1
     * </pre>
     */
    public function lInsert(string $key, string $pos, mixed $pivot, mixed $value): RedisCluster|int|false {}

    /**
     * Return the specified element of the list stored at the specified key.
     * 0 the first element, 1 the second ... -1 the last element, -2 the penultimate ...
     * Return FALSE in case of a bad index or a key that doesn't point to a list.
     *
     * @param string $key
     * @param int    $index
     *
     * @return mixed the element at this index,
     * Bool FALSE if the key identifies a non-string data type, or no value corresponds to this index in the list Key.
     * @throws RedisClusterException
     * @link    https://redis.io/commands/lindex
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');  // key1 => [ 'A', 'B', 'C' ]
     * $redisCluster->lGet('key1', 0);     // 'A'
     * $redisCluster->lGet('key1', -1);    // 'C'
     * $redisCluster->lGet('key1', 10);    // `FALSE`
     * </pre>
     */
    public function lindex(string $key, int $index): mixed {}

    /**
     * Removes the first count occurrences of the value element from the list.
     * If count is zero, all the matching elements are removed. If count is negative,
     * elements are removed from tail to head.
     *
     * @param   string $key
     * @param   mixed $value
     * @param   int    $count
     *
     * @return  RedisCluster|int|bool     the number of elements to remove
     * bool FALSE if the value identified by key is not a list.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lrem
     * @example
     * <pre>
     * $redisCluster->lPush('key1', 'A');
     * $redisCluster->lPush('key1', 'B');
     * $redisCluster->lPush('key1', 'C');
     * $redisCluster->lPush('key1', 'A');
     * $redisCluster->lPush('key1', 'A');
     *
     * $redisCluster->lRange('key1', 0, -1);   // array('A', 'A', 'C', 'B', 'A')
     * $redisCluster->lrem('key1', 'A', 2);    // 2
     * $redisCluster->lRange('key1', 0, -1);   // array('C', 'B', 'A')
     * </pre>
     */
    public function lrem(string $key, mixed $value, int $count = 0): RedisCluster|int|bool {}

    /**
     * A blocking version of rpoplpush, with an integral timeout in the third parameter.
     *
     * @param   string $srckey
     * @param   string $deskey
     * @param   int    $timeout
     *
     * @return  string|false  The element that was moved in case of success, FALSE in case of timeout.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/brpoplpush
     */
    public function brpoplpush(string $srckey, string $deskey, int $timeout): mixed {}

    /**
     * Pops a value from the tail of a list, and pushes it to the front of another list.
     * Also return this value.
     *
     * @param   string $src
     * @param   string $dst
     *
     * @return  RedisCluster|bool|string  The element that was moved in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @since   redis >= 1.2
     *
     * @link    https://redis.io/commands/rpoplpush
     * @example
     * <pre>
     * $redisCluster->del('x', 'y');
     *
     * $redisCluster->lPush('x', 'abc');
     * $redisCluster->lPush('x', 'def');
     * $redisCluster->lPush('y', '123');
     * $redisCluster->lPush('y', '456');
     *
     * // move the last of x to the front of y.
     * var_dump($redisCluster->rpoplpush('x', 'y'));
     * var_dump($redisCluster->lRange('x', 0, -1));
     * var_dump($redisCluster->lRange('y', 0, -1));
     *
     * ////Output:
     * //
     * //string(3) "abc"
     * //array(1) {
     * //  [0]=>
     * //  string(3) "def"
     * //}
     * //array(3) {
     * //  [0]=>
     * //  string(3) "abc"
     * //  [1]=>
     * //  string(3) "456"
     * //  [2]=>
     * //  string(3) "123"
     * //}
     * </pre>
     */
    public function rpoplpush(string $src, string $dst): RedisCluster|bool|string {}

    /**
     * Returns the size of a list identified by Key. If the list didn't exist or is empty,
     * the command returns 0. If the data type identified by Key is not a list, the command return FALSE.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|bool     The size of the list identified by Key exists.
     * bool FALSE if the data type identified by Key is not list
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/llen
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');  // key1 => [ 'A', 'B', 'C' ]
     * $redisCluster->lLen('key1');       // 3
     * $redisCluster->rPop('key1');
     * $redisCluster->lLen('key1');       // 2
     * </pre>
     */
    public function lLen(string $key): RedisCluster|int|bool {}

    /**
     * Returns the set cardinality (number of elements) of the set stored at key.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false   the cardinality (number of elements) of the set, or 0 if key does not exist.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/scard
     * @example
     * <pre>
     * $redisCluster->sAdd('key1' , 'set1');
     * $redisCluster->sAdd('key1' , 'set2');
     * $redisCluster->sAdd('key1' , 'set3');   // 'key1' => {'set1', 'set2', 'set3'}
     * $redisCluster->scard('key1');           // 3
     * $redisCluster->scard('keyX');           // 0
     * </pre>
     */
    public function scard(string $key): RedisCluster|int|false {}

    /**
     * Returns all the members of the set value stored at key.
     * This has the same effect as running SINTER with one argument key.
     *
     * @param   string $key
     *
     * @return  RedisCluster|array|false   All elements of the set.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/smembers
     * @example
     * <pre>
     * $redisCluster->del('s');
     * $redisCluster->sAdd('s', 'a');
     * $redisCluster->sAdd('s', 'b');
     * $redisCluster->sAdd('s', 'a');
     * $redisCluster->sAdd('s', 'c');
     * var_dump($redisCluster->sMembers('s'));
     *
     * ////Output:
     * //
     * //array(3) {
     * //  [0]=>
     * //  string(1) "b"
     * //  [1]=>
     * //  string(1) "c"
     * //  [2]=>
     * //  string(1) "a"
     * //}
     * // The order is random and corresponds to redis' own internal representation of the set structure.
     * </pre>
     */
    public function sMembers(string $key): RedisCluster|array|false {}

    /**
     * Returns if member is a member of the set stored at key.
     *
     * @param   string $key
     * @param   mixed $value
     *
     * @return  RedisCluster|bool    TRUE if value is a member of the set at key key, FALSE otherwise.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sismember
     * @example
     * <pre>
     * $redisCluster->sAdd('key1' , 'set1');
     * $redisCluster->sAdd('key1' , 'set2');
     * $redisCluster->sAdd('key1' , 'set3'); // 'key1' => {'set1', 'set2', 'set3'}
     *
     * $redisCluster->sIsMember('key1', 'set1'); // TRUE
     * $redisCluster->sIsMember('key1', 'setX'); // FALSE
     * </pre>
     */
    public function sismember(string $key, mixed $value): RedisCluster|bool {}

    /**
     * Adds a values to the set value stored at key.
     * If this value is already in the set, FALSE is returned.
     *
     * @param   string $key    Required key
     * @param   mixed $value Required value
     * @param   mixed ...$other_values
     *
     * @return  RedisCluster|int|false     The number of elements added to the set
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sadd
     * @example
     * <pre>
     * $redisCluster->sAdd('k', 'v1');                // int(1)
     * $redisCluster->sAdd('k', 'v1', 'v2', 'v3');    // int(2)
     * </pre>
     */
    public function sAdd(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|false {}

    /**
     * Adds a values to the set value stored at key.
     * If this value is already in the set, FALSE is returned.
     *
     * @param   string $key Required key
     * @param   array  $values
     *
     * @return  RedisCluster|bool|int    The number of elements added to the set
     * @throws  RedisClusterException
     * @example
     * <pre>
     * $redisCluster->sAddArray('k', ['v1', 'v2', 'v3']);
     * //This is a feature in php only. Same as $redisCluster->sAdd('k', 'v1', 'v2', 'v3');
     * </pre>
     */
    public function sAddArray(string $key, array $values): RedisCluster|bool|int {}

    /**
     * Removes the specified members from the set value stored at key.
     *
     * @param   string $key
     * @param   mixed $value
     * @param   mixed ...$other_values
     *
     * @return  RedisCluster|int|false     The number of elements removed from the set.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/srem
     * @example
     * <pre>
     * var_dump( $redisCluster->sAdd('k', 'v1', 'v2', 'v3') );    // int(3)
     * var_dump( $redisCluster->sRem('k', 'v2', 'v3') );          // int(2)
     * var_dump( $redisCluster->sMembers('k') );
     * //// Output:
     * // array(1) {
     * //   [0]=> string(2) "v1"
     * // }
     * </pre>
     */
    public function srem(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|false {}

    /**
     * Performs the union between N sets and returns it.
     *
     * @param   string $key Any number of keys corresponding to sets in redis.
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|bool|array   of strings: The union of all these sets.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sunionstore
     * @example
     * <pre>
     * $redisCluster->del('s0', 's1', 's2');
     *
     * $redisCluster->sAdd('s0', '1');
     * $redisCluster->sAdd('s0', '2');
     * $redisCluster->sAdd('s1', '3');
     * $redisCluster->sAdd('s1', '1');
     * $redisCluster->sAdd('s2', '3');
     * $redisCluster->sAdd('s2', '4');
     *
     * var_dump($redisCluster->sUnion('s0', 's1', 's2'));
     *
     * //// Output:
     * //
     * //array(4) {
     * //  [0]=>
     * //  string(1) "3"
     * //  [1]=>
     * //  string(1) "4"
     * //  [2]=>
     * //  string(1) "1"
     * //  [3]=>
     * //  string(1) "2"
     * //}
     * </pre>
     */
    public function sUnion(string $key, string ...$other_keys): RedisCluster|bool|array {}

    /**
     * Performs the same action as sUnion, but stores the result in the first key
     *
     * @param   string $dst the key to store the diff into.
     * @param   string $key   Any number of keys corresponding to sets in redis.
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|int|false     Any number of keys corresponding to sets in redis.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sunionstore
     * @example
     * <pre>
     * $redisCluster->del('s0', 's1', 's2');
     *
     * $redisCluster->sAdd('s0', '1');
     * $redisCluster->sAdd('s0', '2');
     * $redisCluster->sAdd('s1', '3');
     * $redisCluster->sAdd('s1', '1');
     * $redisCluster->sAdd('s2', '3');
     * $redisCluster->sAdd('s2', '4');
     *
     * var_dump($redisCluster->sUnionStore('dst', 's0', 's1', 's2'));
     * var_dump($redisCluster->sMembers('dst'));
     *
     * //// Output:
     * //
     * //int(4)
     * //array(4) {
     * //  [0]=>
     * //  string(1) "3"
     * //  [1]=>
     * //  string(1) "4"
     * //  [2]=>
     * //  string(1) "1"
     * //  [3]=>
     * //  string(1) "2"
     * //}
     * </pre>
     */
    public function sUnionStore(string $dst, string $key, string ...$other_keys): RedisCluster|int|false {}

    /**
     * Returns the members of a set resulting from the intersection of all the sets
     * held at the specified keys. If just a single key is specified, then this command
     * produces the members of this set. If one of the keys is missing, FALSE is returned.
     *
     * @param   array|string $keykeys identifying the different sets on which we will apply the intersection.
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|array|false contain the result of the intersection between those keys.
     * If the intersection between the different sets is empty, the return value will be empty array.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sinterstore
     * @example
     * <pre>
     * $redisCluster->sAdd('key1', 'val1');
     * $redisCluster->sAdd('key1', 'val2');
     * $redisCluster->sAdd('key1', 'val3');
     * $redisCluster->sAdd('key1', 'val4');
     *
     * $redisCluster->sAdd('key2', 'val3');
     * $redisCluster->sAdd('key2', 'val4');
     *
     * $redisCluster->sAdd('key3', 'val3');
     * $redisCluster->sAdd('key3', 'val4');
     *
     * var_dump($redisCluster->sInter('key1', 'key2', 'key3'));
     *
     * // Output:
     * //
     * //array(2) {
     * //  [0]=>
     * //  string(4) "val4"
     * //  [1]=>
     * //  string(4) "val3"
     * //}
     * </pre>
     */
    public function sInter(array|string $key, string ...$other_keys): RedisCluster|array|false {}

    /**
     * Performs a sInter command and stores the result in a new set.
     *
     * @param   array|string $key the key to store the diff into.
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|int|false    The cardinality of the resulting set, or FALSE in case of a missing key.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sinterstore
     * @example
     * <pre>
     * $redisCluster->sAdd('key1', 'val1');
     * $redisCluster->sAdd('key1', 'val2');
     * $redisCluster->sAdd('key1', 'val3');
     * $redisCluster->sAdd('key1', 'val4');
     *
     * $redisCluster->sAdd('key2', 'val3');
     * $redisCluster->sAdd('key2', 'val4');
     *
     * $redisCluster->sAdd('key3', 'val3');
     * $redisCluster->sAdd('key3', 'val4');
     *
     * var_dump($redisCluster->sInterStore('output', 'key1', 'key2', 'key3'));
     * var_dump($redisCluster->sMembers('output'));
     *
     * //// Output:
     * //
     * //int(2)
     * //array(2) {
     * //  [0]=>
     * //  string(4) "val4"
     * //  [1]=>
     * //  string(4) "val3"
     * //}
     * </pre>
     */
    public function sInterStore(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    /**
     * Performs the difference between N sets and returns it.
     *
     * @param   string $key Any number of keys corresponding to sets in redis.
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|array|false   of strings: The difference of the first set will all the others.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sdiff
     * @example
     * <pre>
     * $redisCluster->del('s0', 's1', 's2');
     *
     * $redisCluster->sAdd('s0', '1');
     * $redisCluster->sAdd('s0', '2');
     * $redisCluster->sAdd('s0', '3');
     * $redisCluster->sAdd('s0', '4');
     *
     * $redisCluster->sAdd('s1', '1');
     * $redisCluster->sAdd('s2', '3');
     *
     * var_dump($redisCluster->sDiff('s0', 's1', 's2'));
     *
     * //// Output:
     * //
     * //array(2) {
     * //  [0]=>
     * //  string(1) "4"
     * //  [1]=>
     * //  string(1) "2"
     * //}
     * </pre>
     */
    public function sDiff(string $key, string ...$other_keys): RedisCluster|array|false {}

    /**
     * Performs the same action as sDiff, but stores the result in the first key
     *
     * @param   string $dst the key to store the diff into.
     * @param   string $key   Any number of keys corresponding to sets in redis
     * @param   string ...$other_keys
     *
     * @return  RedisCluster|int|false    The cardinality of the resulting set, or FALSE in case of a missing key.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sdiffstore
     * @example
     * <pre>
     * $redisCluster->del('s0', 's1', 's2');
     *
     * $redisCluster->sAdd('s0', '1');
     * $redisCluster->sAdd('s0', '2');
     * $redisCluster->sAdd('s0', '3');
     * $redisCluster->sAdd('s0', '4');
     *
     * $redisCluster->sAdd('s1', '1');
     * $redisCluster->sAdd('s2', '3');
     *
     * var_dump($redisCluster->sDiffStore('dst', 's0', 's1', 's2'));
     * var_dump($redisCluster->sMembers('dst'));
     *
     * //// Output:
     * //
     * //int(2)
     * //array(2) {
     * //  [0]=>
     * //  string(1) "4"
     * //  [1]=>
     * //  string(1) "2"
     * //}
     * </pre>
     */
    public function sDiffStore(string $dst, string $key, string ...$other_keys): RedisCluster|int|false {}

    /**
     * Returns a random element(s) from the set value at Key, without removing it.
     *
     * @param   string $key
     * @param   int    $count [optional]
     *
     * @return  RedisCluster|string|array|false  value(s) from the set
     * bool FALSE if set identified by key is empty or doesn't exist and count argument isn't passed.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/srandmember
     * @example
     * <pre>
     * $redisCluster->sAdd('key1' , 'one');
     * $redisCluster->sAdd('key1' , 'two');
     * $redisCluster->sAdd('key1' , 'three');              // 'key1' => {'one', 'two', 'three'}
     *
     * var_dump( $redisCluster->sRandMember('key1') );     // 'key1' => {'one', 'two', 'three'}
     *
     * // string(5) "three"
     *
     * var_dump( $redisCluster->sRandMember('key1', 2) );  // 'key1' => {'one', 'two', 'three'}
     *
     * // array(2) {
     * //   [0]=> string(2) "one"
     * //   [1]=> string(2) "three"
     * // }
     * </pre>
     */
    public function sRandMember(string $key, int $count = 0): RedisCluster|string|array|false {}

    /**
     * Get the length of a string value.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/strlen
     * @example
     * <pre>
     * $redisCluster->set('key', 'value');
     * $redisCluster->strlen('key'); // 5
     * </pre>
     */
    public function strlen(string $key): RedisCluster|int|false {}

    /**
     * Remove the expiration timer from a key.
     *
     * @param   string $key
     *
     * @return  RedisCluster|bool   TRUE if a timeout was removed, FALSE if the key didn’t exist or didn’t have an expiration timer.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/persist
     * @example $redisCluster->persist('key');
     */
    public function persist(string $key): RedisCluster|bool {}

    /**
     * Returns the remaining time to live of a key that has a timeout.
     * This introspection capability allows a Redis client to check how many seconds a given key will continue to be
     * part of the dataset. In Redis 2.6 or older the command returns -1 if the key does not exist or if the key exist
     * but has no associated expire. Starting with Redis 2.8 the return value in case of error changed: Returns -2 if
     * the key does not exist. Returns -1 if the key exists but has no associated expire.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false    the time left to live in seconds.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/ttl
     * @example $redisCluster->ttl('key');
     */
    public function ttl(string $key): RedisCluster|int|false {}

    /**
     * Returns the remaining time to live of a key that has an expire set,
     * with the sole difference that TTL returns the amount of remaining time in seconds while PTTL returns it in
     * milliseconds. In Redis 2.6 or older the command returns -1 if the key does not exist or if the key exist but has
     * no associated expire. Starting with Redis 2.8 the return value in case of error changed: Returns -2 if the key
     * does not exist. Returns -1 if the key exists but has no associated expire.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false     the time left to live in milliseconds.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pttl
     * @example $redisCluster->pttl('key');
     */
    public function pttl(string $key): RedisCluster|int|false {}

    /**
     * Returns the cardinality of an ordered set.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false     the set's cardinality
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zsize
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 0, 'val0');
     * $redisCluster->zAdd('key', 2, 'val2');
     * $redisCluster->zAdd('key', 10, 'val10');
     * $redisCluster->zCard('key');            // 3
     * </pre>
     */
    public function zCard(string $key): RedisCluster|int|false {}

    /**
     * Returns the number of elements of the sorted set stored at the specified key which have
     * scores in the range [start,end]. Adding a parenthesis before start or end excludes it
     * from the range. +inf and -inf are also valid limits.
     *
     * @param   string $key
     * @param   string $start
     * @param   string $end
     *
     * @return  RedisCluster|int|false     the size of a corresponding zRangeByScore.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zcount
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 0, 'val0');
     * $redisCluster->zAdd('key', 2, 'val2');
     * $redisCluster->zAdd('key', 10, 'val10');
     * $redisCluster->zCount('key', 0, 3); // 2, corresponding to array('val0', 'val2')
     * </pre>
     */
    public function zCount(string $key, string $start, string $end): RedisCluster|int|false {}

    /**
     * Deletes the elements of the sorted set stored at the specified key which have scores in the range [start,end].
     *
     * @param   string $key
     * @param   string $start double or "+inf" or "-inf" as a string
     * @param   string $end double or "+inf" or "-inf" as a string
     *
     * @return  RedisCluster|int|false             The number of values deleted from the sorted set
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zremrangebyscore
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 0, 'val0');
     * $redisCluster->zAdd('key', 2, 'val2');
     * $redisCluster->zAdd('key', 10, 'val10');
     * $redisCluster->zRemRangeByScore('key', '0', '3'); // 2
     * </pre>
     */
    public function zRemRangeByScore(string $key, string $min, string $max): RedisCluster|int|false {}

    /**
     * Returns the score of a given member in the specified sorted set.
     *
     * @param   string $key
     * @param   mixed $member
     *
     * @return  RedisCluster|float|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zscore
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 2.5, 'val2');
     * $redisCluster->zScore('key', 'val2'); // 2.5
     * </pre>
     */
    public function zScore(string $key, mixed $member): RedisCluster|float|false {}

    /**
     * Adds the specified member with a given score to the sorted set stored at key.
     *
     * @param   string $key    Required key
     * @param   array|float $score_or_options
     * @param   mixed ...$more_scores_and_mems
     *
     * @return  RedisCluster|int|float|false     Number of values added
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zadd
     * @example
     * <pre>
     * $redisCluster->zAdd('z', 1, 'v2', 2, 'v2', 3, 'v3', 4, 'v4' );  // int(3)
     * $redisCluster->zRem('z', 'v2', 'v3');                           // int(2)
     * var_dump( $redisCluster->zRange('z', 0, -1) );
     *
     * //// Output:
     * // array(1) {
     * //   [0]=> string(2) "v4"
     * // }
     * </pre>
     */
    public function zAdd(string $key, array|float $score_or_options, mixed ...$more_scores_and_mems): RedisCluster|int|float|false {}

    /**
     * Increments the score of a member from a sorted set by a given amount.
     *
     * @param   string $key
     * @param   float  $value (double) value that will be added to the member's score
     * @param   string $member
     *
     * @return  RedisCluster|float|false   the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zincrby
     * @example
     * <pre>
     * $redisCluster->del('key');
     * $redisCluster->zIncrBy('key', 2.5, 'member1');// key or member1 didn't exist, so member1's score is to 0 ;
     *                                              //before the increment and now has the value 2.5
     * $redisCluster->zIncrBy('key', 1, 'member1');    // 3.5
     * </pre>
     */
    public function zIncrBy(string $key, float $value, string $member): RedisCluster|float|false {}

    /**
     * Returns the length of a hash, in number of items
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false     the number of items in a hash, FALSE if the key doesn't exist or isn't a hash.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hlen
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'key1', 'hello');
     * $redisCluster->hSet('h', 'key2', 'plop');
     * $redisCluster->hLen('h'); // returns 2
     * </pre>
     */
    public function hLen(string $key): RedisCluster|int|false {}

    /**
     * Returns the keys in a hash, as an array of strings.
     *
     * @param   string $key
     *
     * @return  RedisCluster|array|false   An array of elements, the keys of the hash. This works like PHP's array_keys().
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hkeys
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'a', 'x');
     * $redisCluster->hSet('h', 'b', 'y');
     * $redisCluster->hSet('h', 'c', 'z');
     * $redisCluster->hSet('h', 'd', 't');
     * var_dump($redisCluster->hKeys('h'));
     *
     * //// Output:
     * //
     * // array(4) {
     * // [0]=>
     * // string(1) "a"
     * // [1]=>
     * // string(1) "b"
     * // [2]=>
     * // string(1) "c"
     * // [3]=>
     * // string(1) "d"
     * // }
     * // The order is random and corresponds to redis' own internal representation of the set structure.
     * </pre>
     */
    public function hKeys(string $key): RedisCluster|array|false {}

    /**
     * Returns the values in a hash, as an array of strings.
     *
     * @param   string $key
     *
     * @return  RedisCluster|array|false   An array of elements, the values of the hash. This works like PHP's array_values().
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hvals
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'a', 'x');
     * $redisCluster->hSet('h', 'b', 'y');
     * $redisCluster->hSet('h', 'c', 'z');
     * $redisCluster->hSet('h', 'd', 't');
     * var_dump($redisCluster->hVals('h'));
     *
     * //// Output:
     * //
     * // array(4) {
     * //   [0]=>
     * //   string(1) "x"
     * //   [1]=>
     * //   string(1) "y"
     * //   [2]=>
     * //   string(1) "z"
     * //   [3]=>
     * //   string(1) "t"
     * // }
     * // The order is random and corresponds to redis' own internal representation of the set structure.
     * </pre>
     */
    public function hVals(string $key): RedisCluster|array|false {}

    /**
     * Gets a value from the hash stored at key.
     * If the hash table doesn't exist, or the key doesn't exist, FALSE is returned.
     *
     * @param   string $key
     * @param   string $member
     *
     * @return  mixed  The value, if the command executed successfully BOOL FALSE in case of failure
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hget
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'a', 'x');
     * $redisCluster->hGet('h', 'a'); // 'X'
     * </pre>
     */
    public function hGet(string $key, string $member): mixed {}

    /**
     * Returns the whole hash, as an array of strings indexed by strings.
     *
     * @param   string $key
     *
     * @return  RedisCluster|array|false   An array of elements, the contents of the hash.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hgetall
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'a', 'x');
     * $redisCluster->hSet('h', 'b', 'y');
     * $redisCluster->hSet('h', 'c', 'z');
     * $redisCluster->hSet('h', 'd', 't');
     * var_dump($redisCluster->hGetAll('h'));
     *
     * //// Output:
     * //
     * // array(4) {
     * //   ["a"]=>
     * //   string(1) "x"
     * //   ["b"]=>
     * //   string(1) "y"
     * //   ["c"]=>
     * //   string(1) "z"
     * //   ["d"]=>
     * //   string(1) "t"
     * // }
     * // The order is random and corresponds to redis' own internal representation of the set structure.
     * </pre>
     */
    public function hGetAll(string $key): RedisCluster|array|false {}

    /**
     * Verify if the specified member exists in a key.
     *
     * @param   string $key
     * @param   string $member
     *
     * @return  RedisCluster|bool   If the member exists in the hash table, return TRUE, otherwise return FALSE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hexists
     * @example
     * <pre>
     * $redisCluster->hSet('h', 'a', 'x');
     * $redisCluster->hExists('h', 'a');               //  TRUE
     * $redisCluster->hExists('h', 'NonExistingKey');  // FALSE
     * </pre>
     */
    public function hExists(string $key, string $member): RedisCluster|bool {}

    /**
     * Increments the value of a member from a hash by a given amount.
     *
     * @param   string $key
     * @param   string $member
     * @param   int    $value (integer) value that will be added to the member's value
     *
     * @return  RedisCluster|int|false     the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hincrby
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hIncrBy('h', 'x', 2); // returns 2: h[x] = 2 now.
     * $redisCluster->hIncrBy('h', 'x', 1); // h[x] ← 2 + 1. Returns 3
     * </pre>
     */
    public function hIncrBy(string $key, string $member, int $value): RedisCluster|int|false {}

    /**
     * Adds a value to the hash stored at key. If this value is already in the hash, FALSE is returned.
     *
     * @param string $key
     * @param string $member
     * @param mixed $value
     *
     * @return RedisCluster|int|false
     * 1 if value didn't exist and was added successfully,
     * 0 if the value was already present and was replaced, FALSE if there was an error.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hset
     * @example
     * <pre>
     * $redisCluster->del('h')
     * $redisCluster->hSet('h', 'key1', 'hello');  // 1, 'key1' => 'hello' in the hash at "h"
     * $redisCluster->hGet('h', 'key1');           // returns "hello"
     *
     * $redisCluster->hSet('h', 'key1', 'plop');   // 0, value was replaced.
     * $redisCluster->hGet('h', 'key1');           // returns "plop"
     * </pre>
     */
    public function hSet(string $key, string $member, mixed $value): RedisCluster|int|false {}

    /**
     * Adds a value to the hash stored at key only if this field isn't already in the hash.
     *
     * @param   string $key
     * @param   string $member
     * @param   string $value
     *
     * @return  RedisCluster|bool    TRUE if the field was set, FALSE if it was already present.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hsetnx
     * @example
     * <pre>
     * $redisCluster->del('h')
     * $redisCluster->hSetNx('h', 'key1', 'hello'); // TRUE, 'key1' => 'hello' in the hash at "h"
     * $redisCluster->hSetNx('h', 'key1', 'world'); // FALSE, 'key1' => 'hello' in the hash at "h". No change since the
     * field wasn't replaced.
     * </pre>
     */
    public function hSetNx(string $key, string $member, mixed $value): RedisCluster|bool {}

    /**
     * Retrieve the values associated to the specified fields in the hash.
     *
     * @param   string $key
     * @param   array  $keys
     *
     * @return  RedisCluster|array|false   Array An array of elements, the values of the specified fields in the hash,
     * with the hash keys as array keys.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hmget
     * @example
     * <pre>
     * $redisCluster->del('h');
     * $redisCluster->hSet('h', 'field1', 'value1');
     * $redisCluster->hSet('h', 'field2', 'value2');
     * $redisCluster->hMget('h', array('field1', 'field2')); // returns array('field1' => 'value1', 'field2' =>
     * 'value2')
     * </pre>
     */
    public function hMget(string $key, array $keys): RedisCluster|array|false {}

    /**
     * Fills in a whole hash. Non-string values are converted to string, using the standard (string) cast.
     * NULL values are stored as empty strings
     *
     * @param   string $key
     * @param   array  $key_values key → value array
     *
     * @return  RedisCluster|bool
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hmset
     * @example
     * <pre>
     * $redisCluster->del('user:1');
     * $redisCluster->hMSet('user:1', array('name' => 'Joe', 'salary' => 2000));
     * $redisCluster->hIncrBy('user:1', 'salary', 100); // Joe earns 100 more now.
     * </pre>
     */
    public function hMset(string $key, array $key_values): RedisCluster|bool {}

    /**
     * Removes a values from the hash stored at key.
     * If the hash table doesn't exist, or the key doesn't exist, FALSE is returned.
     *
     * @param   string $key
     * @param   string $member
     * @param   string ...$other_members
     *
     * @return  RedisCluster|int|false     Number of deleted fields
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hdel
     * @example
     * <pre>
     * $redisCluster->hMSet('h',
     *               array(
     *                    'f1' => 'v1',
     *                    'f2' => 'v2',
     *                    'f3' => 'v3',
     *                    'f4' => 'v4',
     *               ));
     *
     * var_dump( $redisCluster->hDel('h', 'f1') );        // int(1)
     * var_dump( $redisCluster->hDel('h', 'f2', 'f3') );  // int(2)
     *
     * var_dump( $redisCluster->hGetAll('h') );
     *
     * //// Output:
     * //
     * //  array(1) {
     * //    ["f4"]=> string(2) "v4"
     * //  }
     * </pre>
     */
    public function hDel(string $key, string $member, string ...$other_members): RedisCluster|int|false {}

    /**
     * Increment the float value of a hash field by the given amount
     *
     * @param   string $key
     * @param   string $member
     * @param   float  $increment
     *
     * @return  RedisCluster|float|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hincrbyfloat
     * @example
     * <pre>
     * $redisCluster->hset('h', 'float', 3);
     * $redisCluster->hset('h', 'int',   3);
     * var_dump( $redisCluster->hIncrByFloat('h', 'float', 1.5) ); // float(4.5)
     *
     * var_dump( $redisCluster->hGetAll('h') );
     *
     * //// Output:
     * //
     * // array(2) {
     * //   ["float"]=>
     * //   string(3) "4.5"
     * //   ["int"]=>
     * //   string(1) "3"
     * // }
     * </pre>
     */
    public function hIncrByFloat(string $key, string $member, float $value): RedisCluster|float|false {}

    /**
     * Dump a key out of a redis database, the value of which can later be passed into redis using the RESTORE command.
     * The data that comes out of DUMP is a binary representation of the key as Redis stores it.
     *
     * @param   string $key
     *
     * @return  RedisCluster|string|false  The Redis encoded value of the key, or FALSE if the key doesn't exist
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/dump
     * @example
     * <pre>
     * $redisCluster->set('foo', 'bar');
     * $val = $redisCluster->dump('foo'); // $val will be the Redis encoded key value
     * </pre>
     */
    public function dump(string $key): RedisCluster|string|false {}

    /**
     * Returns the rank of a given member in the specified sorted set, starting at 0 for the item
     * with the smallest score. zRevRank starts at 0 for the item with the largest score.
     *
     * @param   string $key
     * @param   mixed $member
     *
     * @return  RedisCluster|int|false     the item's score.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrank
     * @example
     * <pre>
     * $redisCluster->del('z');
     * $redisCluster->zAdd('key', 1, 'one');
     * $redisCluster->zAdd('key', 2, 'two');
     * $redisCluster->zRank('key', 'one');     // 0
     * $redisCluster->zRank('key', 'two');     // 1
     * $redisCluster->zRevRank('key', 'one');  // 1
     * $redisCluster->zRevRank('key', 'two');  // 0
     * </pre>
     */
    public function zRank(string $key, mixed $member): RedisCluster|int|false {}

    /**
     * @param  string $key
     * @param  mixed $member
     *
     * @return RedisCluster|int|false    the item's score
     * @throws RedisClusterException
     * @see    zRank()
     *
     * @link   https://redis.io/commands/zrevrank
     */
    public function zRevRank(string $key, mixed $member): RedisCluster|int|false {}

    /**
     * Increment the number stored at key by one.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false    the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/incr
     * @example
     * <pre>
     * $redisCluster->incr('key1'); // key1 didn't exists, set to 0 before the increment and now has the value 1
     * $redisCluster->incr('key1'); // 2
     * $redisCluster->incr('key1'); // 3
     * $redisCluster->incr('key1'); // 4
     * </pre>
     */
    public function incr(string $key, int $by = 1): RedisCluster|int|false {}

    /**
     * Decrement the number stored at key by one.
     *
     * @param   string $key
     * @param   int $by
     *
     * @return  RedisCluster|int|false    the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/decr
     * @example
     * <pre>
     * $redisCluster->decr('key1'); // key1 didn't exists, set to 0 before the increment and now has the value -1
     * $redisCluster->decr('key1'); // -2
     * $redisCluster->decr('key1'); // -3
     * </pre>
     */
    public function decr(string $key, int $by = 1): RedisCluster|int|false {}

    /**
     * Increment the number stored at key by one. If the second argument is filled, it will be used as the integer
     * value of the increment.
     *
     * @param   string $key   key
     * @param   int    $value value that will be added to key (only for incrBy)
     *
     * @return  RedisCluster|int|false         the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/incrby
     * @example
     * <pre>
     * $redisCluster->incr('key1');        // key1 didn't exists, set to 0 before the increment and now has the value 1
     * $redisCluster->incr('key1');        // 2
     * $redisCluster->incr('key1');        // 3
     * $redisCluster->incr('key1');        // 4
     * $redisCluster->incrBy('key1', 10);  // 14
     * </pre>
     */
    public function incrBy(string $key, int $value): RedisCluster|int|false {}

    /**
     * Decrement the number stored at key by one. If the second argument is filled, it will be used as the integer
     * value of the decrement.
     *
     * @param   string $key
     * @param   int    $value that will be subtracted to key (only for decrBy)
     *
     * @return  RedisCluster|int|false       the new value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/decrby
     * @example
     * <pre>
     * $redisCluster->decr('key1');        // key1 didn't exists, set to 0 before the increment and now has the value -1
     * $redisCluster->decr('key1');        // -2
     * $redisCluster->decr('key1');        // -3
     * $redisCluster->decrBy('key1', 10);  // -13
     * </pre>
     */
    public function decrBy(string $key, int $value): RedisCluster|int|false {}

    /**
     * Increment the float value of a key by the given amount
     *
     * @param   string $key
     * @param   float  $value
     *
     * @return  RedisCluster|float|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/incrbyfloat
     * @example
     * <pre>
     * $redisCluster->set('x', 3);
     * var_dump( $redisCluster->incrByFloat('x', 1.5) );   // float(4.5)
     *
     * var_dump( $redisCluster->get('x') );                // string(3) "4.5"
     * </pre>
     */
    public function incrByFloat(string $key, float $value): RedisCluster|float|false {}

    /**
     * Sets an expiration date (a timeout) on an item.
     *
     * @param   string $key The key that will disappear.
     * @param   int    $timeout The key's remaining Time To Live, in seconds.
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/expire
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $redisCluster->expire('x', 3);  // x will disappear in 3 seconds.
     * sleep(5);                    // wait 5 seconds
     * $redisCluster->get('x');            // will return `FALSE`, as 'x' has expired.
     * </pre>
     */
    public function expire(string $key, int $timeout, ?string $mode = null): RedisCluster|bool {}

    /**
     * Sets an expiration date (a timeout in milliseconds) on an item.
     *
     * @param   string $key The key that will disappear.
     * @param   int    $timeout The key's remaining Time To Live, in milliseconds.
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pexpire
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $redisCluster->pexpire('x', 11500); // x will disappear in 11500 milliseconds.
     * $redisCluster->ttl('x');            // 12
     * $redisCluster->pttl('x');           // 11500
     * </pre>
     */
    public function pexpire(string $key, int $timeout, ?string $mode = null): RedisCluster|bool {}

    /**
     * Sets an expiration date (a timestamp) on an item.
     *
     * @param   string $key       The key that will disappear.
     * @param   int    $timestamp Unix timestamp. The key's date of death, in seconds from Epoch time.
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/expireat
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $now = time();               // current timestamp
     * $redisCluster->expireAt('x', $now + 3); // x will disappear in 3 seconds.
     * sleep(5);                        // wait 5 seconds
     * $redisCluster->get('x');                // will return `FALSE`, as 'x' has expired.
     * </pre>
     */
    public function expireAt(string $key, int $timestamp, ?string $mode = null): RedisCluster|bool {}

    /**
     * Sets an expiration date (a timestamp) on an item. Requires a timestamp in milliseconds
     *
     * @param   string $key       The key that will disappear.
     * @param   int    $timestamp Unix timestamp. The key's date of death, in seconds from Epoch time.
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pexpireat
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $redisCluster->pExpireAt('x', 1555555555005);
     * $redisCluster->ttl('x');                       // 218270121
     * $redisCluster->pttl('x');                      // 218270120575
     * </pre>
     */
    public function pexpireAt(string $key, int $timestamp, ?string $mode = null): RedisCluster|bool {}

    /**
     * Append specified string to the string stored in specified key.
     *
     * @param   string $key
     * @param   mixed $value
     *
     * @return  RedisCluster|bool|int Size of the value after the append
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/append
     * @example
     * <pre>
     * $redisCluster->set('key', 'value1');
     * $redisCluster->append('key', 'value2'); // 12
     * $redisCluster->get('key');              // 'value1value2'
     * </pre>
     */
    public function append(string $key, mixed $value): RedisCluster|bool|int {}

    /**
     * Return a single bit out of a larger string
     *
     * @param   string $key
     * @param   int    $value
     *
     * @return  RedisCluster|int|false    the bit value (0 or 1)
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/getbit
     * @example
     * <pre>
     * $redisCluster->set('key', "\x7f");  // this is 0111 1111
     * $redisCluster->getBit('key', 0);    // 0
     * $redisCluster->getBit('key', 1);    // 1
     * </pre>
     */
    public function getBit(string $key, int $value): RedisCluster|int|false {}

    /**
     * Changes a single bit of a string.
     *
     * @param   string   $key
     * @param   int      $offset
     * @param   bool     $onoff bool or int (1 or 0)
     *
     * @return  RedisCluster|int|false    0 or 1, the value of the bit before it was set.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/setbit
     * @example
     * <pre>
     * $redisCluster->set('key', "*");     // ord("*") = 42 = 0x2f = "0010 1010"
     * $redisCluster->setBit('key', 5, 1); // returns 0
     * $redisCluster->setBit('key', 7, 1); // returns 0
     * $redisCluster->get('key');          // chr(0x2f) = "/" = b("0010 1111")
     * </pre>
     */
    public function setBit(string $key, int $offset, bool $onoff): RedisCluster|int|false {}

    /**
     * Bitwise operation on multiple keys.
     *
     * @param   string $operation either "AND", "OR", "NOT", "XOR"
     * @param   string $deskey    return key
     * @param   string $srckey
     *
     * @return  RedisCluster|bool|int The size of the string stored in the destination key.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/bitop
     * @example
     * <pre>
     * $redisCluster->set('bit1', '1'); // 11 0001
     * $redisCluster->set('bit2', '2'); // 11 0010
     *
     * $redisCluster->bitOp('AND', 'bit', 'bit1', 'bit2'); // bit = 110000
     * $redisCluster->bitOp('OR',  'bit', 'bit1', 'bit2'); // bit = 110011
     * $redisCluster->bitOp('NOT', 'bit', 'bit1', 'bit2'); // bit = 110011
     * $redisCluster->bitOp('XOR', 'bit', 'bit1', 'bit2'); // bit = 11
     * </pre>
     */
    public function bitop(string $operation, string $deskey, string $srckey, string ...$otherkeys): RedisCluster|bool|int {}

    /**
     * Return the position of the first bit set to 1 or 0 in a string. The position is returned, thinking of the
     * string as an array of bits from left to right, where the first byte's most significant bit is at position 0,
     * the second byte's most significant bit is at position 8, and so forth.
     *
     * @param string $key   The key to check (must be a string)
     * @param bool   $bit   Whether to look for an unset (0) or set (1) bit.
     * @param int    $start Where in the string to start looking.
     * @param int    $end   Where in the string to stop looking.
     * @param bool   $bybit If true, Redis will treat $start and $end as BIT values and not bytes, so if start
     *                      was 0 and end was 2, Redis would only search the first two bits.
     *
     * @return  RedisCluster|int|false The command returns the position of the first bit set to 1 or 0 according to the request.
     *                                 If we look for set bits (the bit argument is 1) and the string is empty or composed of just
     *                                 zero bytes, -1 is returned. If we look for clear bits (the bit argument is 0) and the string
     *                                 only contains bit set to 1, the function returns the first bit not part of the string on the
     *                                 right. So if the string is three bytes set to the value 0xff the command BITPOS key 0 will
     *                                 return 24, since up to bit 23 all the bits are 1. Basically, the function considers the right
     *                                 of the string as padded with zeros if you look for clear bits and specify no range or the
     *                                 start argument only. However, this behavior changes if you are looking for clear bits and
     *                                 specify a range with both start and end. If no clear bit is found in the specified range, the
     *                                 function returns -1 as the user specified a clear range and there are no 0 bits in that range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/bitpos
     * @example
     * <pre>
     * $redisCluster->set('key', '\xff\xff');
     * $redisCluster->bitpos('key', 1); // int(0)
     * $redisCluster->bitpos('key', 1, 1); // int(8)
     * $redisCluster->bitpos('key', 1, 3); // int(-1)
     * $redisCluster->bitpos('key', 0); // int(16)
     * $redisCluster->bitpos('key', 0, 1); // int(16)
     * $redisCluster->bitpos('key', 0, 1, 5); // int(-1)
     * </pre>
     */
    public function bitpos(string $key, bool $bit, int $start = 0, int $end = -1, bool $bybit = false): RedisCluster|int|false {}

    /**
     * Count bits in a string.
     *
     * @param   string $key
     *
     * @return  RedisCluster|bool|int The number of bits set to 1 in the value behind the input key.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/bitcount
     * @example
     * <pre>
     * $redisCluster->set('bit', '345'); // // 11 0011  0011 0100  0011 0101
     * var_dump( $redisCluster->bitCount('bit', 0, 0) ); // int(4)
     * var_dump( $redisCluster->bitCount('bit', 1, 1) ); // int(3)
     * var_dump( $redisCluster->bitCount('bit', 2, 2) ); // int(4)
     * var_dump( $redisCluster->bitCount('bit', 0, 2) ); // int(11)
     * </pre>
     */
    public function bitcount(string $key, int $start = 0, int $end = -1, bool $bybit = false): RedisCluster|bool|int {}

    /**
     * @param   string $key
     * @param   int    $index
     *
     * @throws  RedisClusterException
     * @see     lIndex()
     *
     * @link    https://redis.io/commands/lindex
     */
    public function lGet(string $key, int $index): RedisCluster|string|bool {}

    /**
     * Return a substring of a larger string
     *
     * @param   string $key
     * @param   int    $start
     * @param   int    $end
     *
     * @return  RedisCluster|string|false the substring
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/getrange
     * @example
     * <pre>
     * $redisCluster->set('key', 'string value');
     * $redisCluster->getRange('key', 0, 5);   // 'string'
     * $redisCluster->getRange('key', -5, -1); // 'value'
     * </pre>
     */
    public function getRange(string $key, int $start, int $end): RedisCluster|string|false {}

    /**
     * Trims an existing list so that it will contain only a specified range of elements.
     *
     * @param string $key
     * @param int    $start
     * @param int    $stop
     *
     * @return RedisCluster|bool    Bool return FALSE if the key identify a non-list value.
     * @throws RedisClusterException
     * @link        https://redis.io/commands/ltrim
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');
     * $redisCluster->lRange('key1', 0, -1); // array('A', 'B', 'C')
     * $redisCluster->ltrim('key1', 0, 1);
     * $redisCluster->lRange('key1', 0, -1); // array('A', 'B')
     * </pre>
     */
    public function ltrim(string $key, int $start, int $end): RedisCluster|bool {}

    /**
     * Returns the specified elements of the list stored at the specified key in
     * the range [start, end]. start and stop are interpretated as indices: 0 the first element,
     * 1 the second ... -1 the last element, -2 the penultimate ...
     *
     * @param   string $key
     * @param   int    $start
     * @param   int    $end
     *
     * @return  RedisCluster|array|false containing the values in specified range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lrange
     * @example
     * <pre>
     * $redisCluster->rPush('key1', 'A');
     * $redisCluster->rPush('key1', 'B');
     * $redisCluster->rPush('key1', 'C');
     * $redisCluster->lrange('key1', 0, -1); // array('A', 'B', 'C')
     * </pre>
     */
    public function lrange(string $key, int $start, int $end): RedisCluster|array|false {}

    /**
     * Deletes the elements of the sorted set stored at the specified key which have rank in the range [start,end].
     *
     * @param   string $key
     * @param   int    $start
     * @param   int    $end
     *
     * @return  RedisCluster|int|false     The number of values deleted from the sorted set
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zremrangebyrank
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 1, 'one');
     * $redisCluster->zAdd('key', 2, 'two');
     * $redisCluster->zAdd('key', 3, 'three');
     * $redisCluster->zRemRangeByRank('key', 0, 1); // 2
     * $redisCluster->zRange('key', 0, -1, true); // array('three' => 3)
     * </pre>
     */
    public function zRemRangeByRank(string $key, string $min, string $max): RedisCluster|int|false {}

    /**
     * Publish messages to channels. Warning: this function will probably change in the future.
     *
     * @param   string $channel a channel to publish to
     * @param   string $message string
     *
     * @return  RedisCluster|bool|int Number of clients that received the message
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/publish
     * @example $redisCluster->publish('chan-1', 'hello, world!'); // send message.
     */
    public function publish(string $channel, string $message): RedisCluster|bool|int {}

    /**
     * Renames a key.
     *
     * @param   string $key_src
     * @param   string $key_dst
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/rename
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $redisCluster->rename('x', 'y');
     * $redisCluster->get('y');   // → 42
     * $redisCluster->get('x');   // → `FALSE`
     * </pre>
     */
    public function rename(string $key_src, string $key_dst): RedisCluster|bool {}

    /**
     * Renames a key.
     *
     * Same as rename, but will not replace a key if the destination already exists.
     * This is the same behaviour as setNx.
     *
     * @param   string $key
     * @param   string $newkey
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/renamenx
     * @example
     * <pre>
     * $redisCluster->set('x', '42');
     * $redisCluster->renameNx('x', 'y');
     * $redisCluster->get('y');   // → 42
     * $redisCluster->get('x');   // → `FALSE`
     * </pre>
     */
    public function renameNx(string $key, string $newkey): RedisCluster|bool {}

    /**
     * When called with a single key, returns the approximated cardinality computed by the HyperLogLog data
     * structure stored at the specified variable, which is 0 if the variable does not exist.
     *
     * @param   string $key
     *
     * @return  RedisCluster|int|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pfcount
     * @example
     * <pre>
     * $redisCluster->pfadd('key1', array('elem1', 'elem2'));
     * $redisCluster->pfadd('key2', array('elem3', 'elem2'));
     * $redisCluster->pfcount('key1'); // int(2)
     * $redisCluster->pfcount(array('key1', 'key2')); // int(3)
     * </pre>
     */
    public function pfcount(string $key): RedisCluster|int|false {}

    /**
     * Adds all the element arguments to the HyperLogLog data structure stored at the key.
     *
     * @param   string $key
     * @param   array  $elements
     *
     * @return  RedisCluster|bool
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pfadd
     * @example $redisCluster->pfadd('key', array('elem1', 'elem2'))
     */
    public function pfadd(string $key, array $elements): RedisCluster|bool {}

    /**
     * Merge multiple HyperLogLog values into an unique value that will approximate the cardinality
     * of the union of the observed Sets of the source HyperLogLog structures.
     *
     * @param   string $key
     * @param   array  $keys
     *
     * @return  RedisCluster|bool
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pfmerge
     * @example
     * <pre>
     * $redisCluster->pfadd('key1', array('elem1', 'elem2'));
     * $redisCluster->pfadd('key2', array('elem3', 'elem2'));
     * $redisCluster->pfmerge('key3', array('key1', 'key2'));
     * $redisCluster->pfCount('key3'); // int(3)
     * </pre>
     */
    public function pfmerge(string $key, array $keys): RedisCluster|bool {}

    /**
     * Changes a substring of a larger string.
     *
     * @param   string $key
     * @param   int    $offset
     * @param   string $value
     *
     * @return  RedisCluster|int|false the length of the string after it was modified.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/setrange
     * @example
     * <pre>
     * $redisCluster->set('key', 'Hello world');
     * $redisCluster->setRange('key', 6, "redis"); // returns 11
     * $redisCluster->get('key');                  // "Hello redis"
     * </pre>
     */
    public function setRange(string $key, int $offset, string $value): RedisCluster|int|false {}

    /**
     * Restore a key from the result of a DUMP operation.
     *
     * @param   string $key   The key name
     * @param   int    $timeout   How long the key should live (if zero, no expire will be set on the key)
     * @param   string $value (binary).  The Redis encoded key value (from DUMP)
     *
     * @return  RedisCluster|bool
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/restore
     * @example
     * <pre>
     * $redisCluster->set('foo', 'bar');
     * $val = $redisCluster->dump('foo');
     * $redisCluster->restore('bar', 0, $val); // The key 'bar', will now be equal to the key 'foo'
     * </pre>
     */
    public function restore(string $key, int $timeout, string $value, ?array $options = null): RedisCluster|bool {}

    /**
     * Moves the specified member from the set at srcKey to the set at dstKey.
     *
     * @param   string $src
     * @param   string $dst
     * @param   string $member
     *
     * @return  RedisCluster|bool    If the operation is successful, return TRUE.
     * If the srcKey and/or dstKey didn't exist, and/or the member didn't exist in srcKey, FALSE is returned.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/smove
     * @example
     * <pre>
     * $redisCluster->sAdd('key1' , 'set11');
     * $redisCluster->sAdd('key1' , 'set12');
     * $redisCluster->sAdd('key1' , 'set13');          // 'key1' => {'set11', 'set12', 'set13'}
     * $redisCluster->sAdd('key2' , 'set21');
     * $redisCluster->sAdd('key2' , 'set22');          // 'key2' => {'set21', 'set22'}
     * $redisCluster->sMove('key1', 'key2', 'set13');  // 'key1' =>  {'set11', 'set12'}
     *                                          // 'key2' =>  {'set21', 'set22', 'set13'}
     * </pre>
     */
    public function sMove(string $src, string $dst, string $member): RedisCluster|bool {}

    /**
     * Returns a range of elements from the ordered set stored at the specified key,
     * with values in the range [start, end]. start and stop are interpreted as zero-based indices:
     * 0 the first element,
     * 1 the second ...
     * -1 the last element,
     * -2 the penultimate ...
     *
     * @param   string $key
     * @param   mixed  $start
     * @param   mixed  $end
     * @param   array|bool|null $options
     *
     * @return  edisCluster|array|bool   Array containing the values in specified range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrange
     * @example
     * <pre>
     * $redisCluster->zAdd('key1', 0, 'val0');
     * $redisCluster->zAdd('key1', 2, 'val2');
     * $redisCluster->zAdd('key1', 10, 'val10');
     * $redisCluster->zRange('key1', 0, -1); // array('val0', 'val2', 'val10')
     * // with scores
     * $redisCluster->zRange('key1', 0, -1, true); // array('val0' => 0, 'val2' => 2, 'val10' => 10)
     * </pre>
     */
    public function zRange(string $key, mixed $start, mixed $end, array|bool|null $options = null): RedisCluster|array|bool {}

    /**
     * Returns the elements of the sorted set stored at the specified key in the range [start, end]
     * in reverse order. start and stop are interpretated as zero-based indices:
     * 0 the first element,
     * 1 the second ...
     * -1 the last element,
     * -2 the penultimate ...
     *
     * @param   string $key
     * @param   string $start
     * @param   string $end
     * @param   null|array $options
     *
     * @return  RedisCluster|bool|array   Array containing the values in specified range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrevrange
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 0, 'val0');
     * $redisCluster->zAdd('key', 2, 'val2');
     * $redisCluster->zAdd('key', 10, 'val10');
     * $redisCluster->zRevRange('key', 0, -1); // array('val10', 'val2', 'val0')
     *
     * // with scores
     * $redisCluster->zRevRange('key', 0, -1, true); // array('val10' => 10, 'val2' => 2, 'val0' => 0)
     * </pre>
     */
    public function zRevRange(string $key, string $min, string $max, ?array $options = null): RedisCluster|bool|array {}

    /**
     * Returns the elements of the sorted set stored at the specified key which have scores in the
     * range [start,end]. Adding a parenthesis before start or end excludes it from the range.
     * +inf and -inf are also valid limits.
     *
     * zRevRangeByScore returns the same items in reverse order, when the start and end parameters are swapped.
     *
     * @param   string $key
     * @param   string $start
     * @param   string $end
     * @param   array  $options Two options are available:
     *                          - withscores => TRUE,
     *                          - and limit => array($offset, $count)
     *
     * @return  RedisCluster|array|false   Array containing the values in specified range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrangebyscore
     * @example
     * <pre>
     * $redisCluster->zAdd('key', 0, 'val0');
     * $redisCluster->zAdd('key', 2, 'val2');
     * $redisCluster->zAdd('key', 10, 'val10');
     * $redisCluster->zRangeByScore('key', 0, 3);
     * // array('val0', 'val2')
     * $redisCluster->zRangeByScore('key', 0, 3, array('withscores' => TRUE);
     * // array('val0' => 0, 'val2' => 2)
     * $redisCluster->zRangeByScore('key', 0, 3, array('limit' => array(1, 1));
     * // array('val2' => 2)
     * $redisCluster->zRangeByScore('key', 0, 3, array('limit' => array(1, 1));
     * // array('val2')
     * $redisCluster->zRangeByScore('key', 0, 3, array('withscores' => TRUE, 'limit' => array(1, 1));
     * // array('val2'=> 2)
     * </pre>
     */
    public function zRangeByScore(string $key, string $start, string $end, array $options = []): RedisCluster|array|false {}

    /**
     * @param   string $key
     * @param   string $start
     * @param   string $end
     * @param   null|array  $options
     *
     * @return  array
     * @throws  RedisClusterException
     * @see     zRangeByScore()
     */
    public function zRevRangeByScore(string $key, string $min, string $max, ?array $options = null): RedisCluster|bool|array {}

    /**
     * Returns a range of members in a sorted set, by lexicographical range
     *
     * @param   string $key    The ZSET you wish to run against.
     * @param   string $min    The minimum alphanumeric value you wish to get.
     * @param   string $max    The maximum alphanumeric value you wish to get.
     * @param   int    $offset Optional argument if you wish to start somewhere other than the first element.
     * @param   int    $count  Optional argument if you wish to limit the number of elements returned.
     *
     * @return  RedisCluster|array|false   Array containing the values in the specified range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrangebylex
     * @example
     * <pre>
     * foreach (array('a', 'b', 'c', 'd', 'e', 'f', 'g') as $k => $char) {
     *     $redisCluster->zAdd('key', $k, $char);
     * }
     *
     * $redisCluster->zRangeByLex('key', '-', '[c'); // array('a', 'b', 'c')
     * $redisCluster->zRangeByLex('key', '-', '(c'); // array('a', 'b')
     * $redisCluster->zRevRangeByLex('key', '(c','-'); // array('b', 'a')
     * </pre>
     */
    public function zRangeByLex(string $key, string $min, string $max, int $offset = -1, int $count = -1): RedisCluster|array|false {}

    /**
     * @param   string $key
     * @param   string $min
     * @param   string $max
     * @param   null|array $options
     *
     * @return  RedisCluster|bool|array
     * @throws  RedisClusterException
     * @see     zRangeByLex()
     *
     * @link    https://redis.io/commands/zrevrangebylex
     */
    public function zRevRangeByLex(string $key, string $min, string $max, ?array $options = null): RedisCluster|bool|array {}

    /**
     * Count the number of members in a sorted set between a given lexicographical range.
     *
     * @param   string $key
     * @param   string $min
     * @param   string $max
     *
     * @return  RedisCluster|int|false The number of elements in the specified score range.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zlexcount
     * @example
     * <pre>
     * foreach (array('a', 'b', 'c', 'd', 'e', 'f', 'g') as $k => $char) {
     *     $redisCluster->zAdd('key', $k, $char);
     * }
     * $redisCluster->zLexCount('key', '[b', '[f'); // 5
     * </pre>
     */
    public function zLexCount(string $key, string $min, string $max): RedisCluster|int|false {}

    /**
     * Remove all members in a sorted set between the given lexicographical range.
     *
     * @param  string  $key  The ZSET you wish to run against.
     * @param  string  $min  The minimum alphanumeric value you wish to get.
     * @param  string  $max  The maximum alphanumeric value you wish to get.
     *
     * @return  RedisCluster|int|false   the number of elements removed.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zremrangebylex
     * @example
     * <pre>
     * foreach (array('a', 'b', 'c', 'd', 'e', 'f', 'g') as $k => $char) {
     *     $redisCluster->zAdd('key', $k, $char);
     * }
     * $redisCluster->zRemRangeByLex('key', '(b','[d'); // 2 , remove element 'c' and 'd'
     * $redisCluster->zRange('key',0,-1);// array('a','b','e','f','g')
     * </pre>
     */
    public function zRemRangeByLex(string $key, string $min, string $max): RedisCluster|int|false {}

    /**
     * Add multiple sorted sets and store the resulting sorted set in a new key
     *
     * @param string $dst
     * @param array  $keys
     * @param null|array $weights
     * @param null|string $aggregate  Either "SUM", "MIN", or "MAX": defines the behaviour to use on
     *                                  duplicate entries during the zUnion.
     *
     * @return RedisCluster|int|false The number of values in the new sorted set.
     * @throws RedisClusterException
     * @link   https://redis.io/commands/zunionstore
     * @example
     * <pre>
     * $redisCluster->del('k1');
     * $redisCluster->del('k2');
     * $redisCluster->del('k3');
     * $redisCluster->del('ko1');
     * $redisCluster->del('ko2');
     * $redisCluster->del('ko3');
     *
     * $redisCluster->zAdd('k1', 0, 'val0');
     * $redisCluster->zAdd('k1', 1, 'val1');
     *
     * $redisCluster->zAdd('k2', 2, 'val2');
     * $redisCluster->zAdd('k2', 3, 'val3');
     *
     * $redisCluster->zunionstore('ko1', array('k1', 'k2')); // 4, 'ko1' => array('val0', 'val1', 'val2', 'val3')
     *
     * // Weighted zunionstore
     * $redisCluster->zunionstore('ko2', array('k1', 'k2'), array(1, 1)); // 4, 'ko2' => array('val0', 'val1', 'val2','val3')
     * $redisCluster->zunionstore('ko3', array('k1', 'k2'), array(5, 1)); // 4, 'ko3' => array('val0', 'val2', 'val3','val1')
     * </pre>
     */
    public function zunionstore(string $dst, array $keys, ?array $weights = null, ?string $aggregate = null): RedisCluster|int|false {}

    /**
     * Intersect multiple sorted sets and store the resulting sorted set in a new key
     *
     * @param   string $dst
     * @param   array  $keys
     * @param   null|array $weights
     * @param   null|string $aggregate Either "SUM", "MIN", or "MAX":
     *                                    defines the behaviour to use on duplicate entries during the zInterStore.
     *
     * @return  RedisCluster|int|false     The number of values in the new sorted set.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zinterstore
     * @example
     * <pre>
     * $redisCluster->del('k1');
     * $redisCluster->del('k2');
     * $redisCluster->del('k3');
     *
     * $redisCluster->del('ko1');
     * $redisCluster->del('ko2');
     * $redisCluster->del('ko3');
     * $redisCluster->del('ko4');
     *
     * $redisCluster->zAdd('k1', 0, 'val0');
     * $redisCluster->zAdd('k1', 1, 'val1');
     * $redisCluster->zAdd('k1', 3, 'val3');
     *
     * $redisCluster->zAdd('k2', 2, 'val1');
     * $redisCluster->zAdd('k2', 3, 'val3');
     *
     * $redisCluster->zInterStore('ko1', array('k1', 'k2'));               // 2, 'ko1' => array('val1', 'val3')
     * $redisCluster->zInterStore('ko2', array('k1', 'k2'), array(1, 1));  // 2, 'ko2' => array('val1', 'val3')
     *
     * // Weighted zInterStore
     * $redisCluster->zInterStore('ko3', array('k1', 'k2'), array(1, 5), 'min'); // 2, 'ko3' => array('val1', 'val3')
     * $redisCluster->zInterStore('ko4', array('k1', 'k2'), array(1, 5), 'max'); // 2, 'ko4' => array('val3', 'val1')
     * </pre>
     */
    public function zinterstore(string $dst, array $keys, ?array $weights = null, ?string $aggregate = null): RedisCluster|int|false {}

    /**
     * Deletes a specified member from the ordered set.
     *
     * @param   string $key
     * @param   string $value
     * @param   string ...$other_values
     *
     * @return  RedisCluster|int|false     Number of deleted values
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zrem
     * @example
     * <pre>
     * $redisCluster->zAdd('z', 1, 'v1', 2, 'v2', 3, 'v3', 4, 'v4' );  // int(2)
     * $redisCluster->zRem('z', 'v2', 'v3');                           // int(2)
     * var_dump( $redisCluster->zRange('z', 0, -1) );
     * //// Output:
     * //
     * // array(2) {
     * //   [0]=> string(2) "v1"
     * //   [1]=> string(2) "v4"
     * // }
     * </pre>
     */
    public function zRem(string $key, string $value, string ...$other_values): RedisCluster|int|false {}

    /**
     * Sort
     *
     * @param   string $key
     * @param   null|array $options  $option array(key => value, ...) - optional, with the following keys and values:
     *                         - 'by' => 'some_pattern_*',
     *                         - 'limit' => array(0, 1),
     *                         - 'get' => 'some_other_pattern_*' or an array of patterns,
     *                         - 'sort' => 'asc' or 'desc',
     *                         - 'alpha' => TRUE,
     *                         - 'store' => 'external-key'
     *
     * @return  RedisCluster|array|bool|int|string
     * @throws  RedisClusterException
     * An array of values, or a number corresponding to the number of elements stored if that was used.
     * @link    https://redis.io/commands/sort
     * @example
     * <pre>
     * $redisCluster->del('s');
     * $redisCluster->sadd('s', 5);
     * $redisCluster->sadd('s', 4);
     * $redisCluster->sadd('s', 2);
     * $redisCluster->sadd('s', 1);
     * $redisCluster->sadd('s', 3);
     *
     * var_dump($redisCluster->sort('s')); // 1,2,3,4,5
     * var_dump($redisCluster->sort('s', array('sort' => 'desc'))); // 5,4,3,2,1
     * var_dump($redisCluster->sort('s', array('sort' => 'desc', 'store' => 'out'))); // (int)5
     * </pre>
     */
    public function sort(string $key, ?array $options = null): RedisCluster|array|bool|int|string {}

    /**
     * Describes the object pointed to by a key.
     * The information to retrieve (string) and the key (string).
     * Info can be one of the following:
     * - "encoding"
     * - "refcount"
     * - "idletime"
     *
     * @param   string $subcommand
     * @param   string $key
     *
     * @return  RedisCluster|int|string|false  for "encoding", int for "refcount" and "idletime", FALSE if the key doesn't exist.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/object
     * @example
     * <pre>
     * $redisCluster->object("encoding", "l"); // → ziplist
     * $redisCluster->object("refcount", "l"); // → 1
     * $redisCluster->object("idletime", "l"); // → 400 (in seconds, with a precision of 10 seconds).
     * </pre>
     */
    public function object(string $subcommand, string $key): RedisCluster|int|string|false {}

    /**
     * Subscribe to channels. Warning: this function will probably change in the future.
     *
     * @param array        $channels an array of channels to subscribe to
     * @param callable    $cb either a string or an array($instance, 'method_name').
     *                                 The callback function receives 3 parameters: the redis instance, the channel
     *                                 name, and the message.
     *
     * @return void            Any non-null return value in the callback will be returned to the caller.
     * @throws RedisClusterException
     * @link    https://redis.io/commands/subscribe
     * @example
     * <pre>
     * function f($redisCluster, $chan, $msg) {
     *  switch($chan) {
     *      case 'chan-1':
     *          ...
     *          break;
     *
     *      case 'chan-2':
     *                     ...
     *          break;
     *
     *      case 'chan-2':
     *          ...
     *          break;
     *      }
     * }
     *
     * $redisCluster->subscribe(array('chan-1', 'chan-2', 'chan-3'), 'f'); // subscribe to 3 chans
     * </pre>
     */
    public function subscribe(array $channels, callable $cb): void {}

    /**
     * Subscribe to channels by pattern
     *
     * @param   array        $patterns     The number of elements removed from the set.
     * @param   callable $callback         Either a string or an array with an object and method.
     *                                       The callback will get four arguments ($redis, $pattern, $channel, $message)
     *
     * @return  void           Any non-null return value in the callback will be returned to the caller.
     * @throws  RedisClusterException
     *
     * @link    https://redis.io/commands/psubscribe
     * @example
     * <pre>
     * function psubscribe($redisCluster, $pattern, $chan, $msg) {
     *  echo "Pattern: $pattern\n";
     *  echo "Channel: $chan\n";
     *  echo "Payload: $msg\n";
     * }
     * </pre>
     */
    public function psubscribe(array $patterns, callable $callback): void {}

    /**
     * Unsubscribes the client from the given channels, or from all of them if none is given.
     *
     * @param array $channels
     *
     * @throws RedisClusterException
     */
    public function unsubscribe(array $channels): bool|array {}

    /**
     * Unsubscribes the client from the given patterns, or from all of them if none is given.
     *
     * @param string $pattern
     * @param string ...$other_patterns
     *
     * @throws RedisClusterException
     */
    public function punsubscribe(string $pattern, string ...$other_patterns): bool|array {}

    /**
     * Evaluate a LUA script serverside, from the SHA1 hash of the script instead of the script itself.
     * In order to run this command Redis will have to have already loaded the script, either by running it or via
     * the SCRIPT LOAD command.
     *
     * @param   string $script_sha
     * @param   array  $args
     * @param   int    $num_keys
     *
     * @return  mixed   @see eval()
     * @throws  RedisClusterException
     * @see     eval()
     * @link    https://redis.io/commands/evalsha
     * @example
     * <pre>
     * $script = 'return 1';
     * $sha = $redisCluster->script('load', $script);
     * $redisCluster->evalsha($sha); // Returns 1
     * </pre>
     */
    public function evalsha(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    /**
     * Scan the keyspace for keys.
     *
     * @param  null|int|string &$iterator          &$iterator Iterator, initialized to NULL.
     * @param  string|array $key_or_address      Node identified by key or host/port array
     * @param  null|string       $pattern   Pattern to match.
     * @param  int          $count     Count of keys per iteration (only a suggestion to Redis).
     *
     * @return bool|array             This function will return an array of keys or FALSE if there are no more keys.
     * @throws RedisClusterException
     * @link   https://redis.io/commands/scan
     * @example
     * <pre>
     * $iterator = null;
     * while($keys = $redisCluster->scan($iterator)) {
     *     foreach($keys as $key) {
     *         echo $key . PHP_EOL;
     *     }
     * }
     * </pre>
     */
    public function scan(null|int|string &$iterator, string|array $key_or_address, ?string $pattern = null, int $count = 0): bool|array {}

    /**
     * Scan a set for members.
     *
     * @param   string $key      The set to search.
     * @param   null|int|string &$iterator    &$iterator LONG (reference) to the iterator as we go.
     * @param   null|string   $pattern  String, optional pattern to match against.
     * @param   int    $count    How many members to return at a time (Redis might return a different amount).
     *
     * @return  array|false   PHPRedis will return an array of keys or FALSE when we're done iterating.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/sscan
     * @example
     * <pre>
     * $iterator = null;
     * while ($members = $redisCluster->sscan('set', $iterator)) {
     *     foreach ($members as $member) {
     *         echo $member . PHP_EOL;
     *     }
     * }
     * </pre>
     */
    public function sscan(string $key, null|int|string &$iterator, ?string $pattern = null, int $count = 0): array|false {}

    /**
     * Scan a sorted set for members, with optional pattern and count.
     *
     * @param   string $key      String, the set to scan.
     * @param   null|int|string &$iterator Long (reference), initialized to NULL.
     * @param   null|string $pattern  String (optional), the pattern to match.
     * @param   int    $count    How many keys to return per iteration (Redis might return a different number).
     *
     * @return  RedisCluster|bool|array   PHPRedis will return matching keys from Redis, or FALSE when iteration is complete.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/zscan
     * @example
     * <pre>
     * $iterator = null;
     * while ($members = $redis->zscan('zset', $iterator)) {
     *     foreach ($members as $member => $score) {
     *         echo $member . ' => ' . $score . PHP_EOL;
     *     }
     * }
     * </pre>
     */
    public function zscan(string $key, null|int|string &$iterator, ?string $pattern = null, int $count = 0): RedisCluster|bool|array {}

    /**
     * Scan a HASH value for members, with an optional pattern and count.
     *
     * @param   string $key
     * @param   null|int|string    &$iterator
     * @param   null|string $pattern Optional pattern to match against.
     * @param   int    $count   How many keys to return in a go (only a sugestion to Redis).
     *
     * @return  array|bool     An array of members that match our pattern.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/hscan
     * @example
     * <pre>
     * $iterator = null;
     * while($elements = $redisCluster->hscan('hash', $iterator)) {
     *    foreach($elements as $key => $value) {
     *         echo $key . ' => ' . $value . PHP_EOL;
     *     }
     * }
     * </pre>
     */
    public function hscan(string $key, null|int|string &$iterator, ?string $pattern = null, int $count = 0): array|bool {}

    /**
     * Detect whether we're in ATOMIC/MULTI/PIPELINE mode.
     *
     * @return  int     Either RedisCluster::ATOMIC, RedisCluster::MULTI or RedisCluster::PIPELINE
     * @throws  RedisClusterException
     * @example $redisCluster->getMode();
     */
    public function getMode(): int {}

    /**
     * The last error message (if any)
     *
     * @return  string|null  A string with the last returned script based error message, or NULL if there is no error
     * @throws  RedisClusterException
     * @example
     * <pre>
     * $redisCluster->eval('this-is-not-lua');
     * $err = $redisCluster->getLastError();
     * // "ERR Error compiling script (new function): user_script:1: '=' expected near '-'"
     * </pre>
     */
    public function getLastError(): string|null {}

    /**
     * Clear the last error message
     *
     * @return bool true
     * @example
     * <pre>
     * $redisCluster->set('x', 'a');
     * $redisCluster->incr('x');
     * $err = $redisCluster->getLastError();
     * // "ERR value is not an integer or out of range"
     * $redisCluster->clearLastError();
     * $err = $redisCluster->getLastError();
     * // NULL
     * </pre>
     */
    public function clearLastError(): bool {}

    /**
     * Get client option
     *
     * @param   int $option parameter
     *
     * @return  int|string     Parameter value.
     * @example
     * // return RedisCluster::SERIALIZER_NONE, RedisCluster::SERIALIZER_PHP, or RedisCluster::SERIALIZER_IGBINARY.
     * $redisCluster->getOption(RedisCluster::OPT_SERIALIZER);
     */
    public function getOption(int $option): mixed {}

    /**
     * Set client option.
     *
     * @param   int        $option parameter
     * @param   mixed      $value  parameter value
     *
     * @return  bool   TRUE on success, FALSE on error.
     * @example
     * <pre>
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_NONE);        // don't serialize data
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_PHP);         // use built-in serialize/unserialize
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_IGBINARY);    // use igBinary serialize/unserialize
     * $redisCluster->setOption(RedisCluster::OPT_PREFIX, 'myAppName:');                             // use custom prefix on all keys
     * </pre>
     */
    public function setOption(int $option, mixed $value): bool {}

    /**
     * A utility method to prefix the key with the prefix setting for phpredis.
     *
     * @param   mixed $key The key you wish to prefix
     *
     * @return  string  If a prefix is set up, the key now prefixed.  If there is no prefix, the key will be returned unchanged.
     * @example
     * <pre>
     * $redisCluster->setOption(RedisCluster::OPT_PREFIX, 'my-prefix:');
     * $redisCluster->_prefix('my-key'); // Will return 'my-prefix:my-key'
     * </pre>
     */
    public function _prefix(string $key): bool|string {}

    /**
     * A utility method to serialize values manually. This method allows you to serialize a value with whatever
     * serializer is configured, manually. This can be useful for serialization/unserialization of data going in
     * and out of EVAL commands as phpredis can't automatically do this itself.  Note that if no serializer is
     * set, phpredis will change Array values to 'Array', and Objects to 'Object'.
     *
     * @param   mixed $value The value to be serialized.
     *
     * @return  mixed
     * @example
     * <pre>
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_NONE);
     * $redisCluster->_serialize("foo"); // returns "foo"
     * $redisCluster->_serialize(Array()); // Returns "Array"
     * $redisCluster->_serialize(new stdClass()); // Returns "Object"
     *
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_PHP);
     * $redisCluster->_serialize("foo"); // Returns 's:3:"foo";'
     * </pre>
     */
    public function _serialize(mixed $value): bool|string {}

    /**
     * A utility method to unserialize data with whatever serializer is set up.  If there is no serializer set, the
     * value will be returned unchanged.  If there is a serializer set up, and the data passed in is malformed, an
     * exception will be thrown. This can be useful if phpredis is serializing values, and you return something from
     * redis in a LUA script that is serialized.
     *
     * @param   string $value The value to be unserialized
     *
     * @return mixed
     * @example
     * <pre>
     * $redisCluster->setOption(RedisCluster::OPT_SERIALIZER, RedisCluster::SERIALIZER_PHP);
     * $redisCluster->_unserialize('a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}'); // Will return Array(1,2,3)
     * </pre>
     */
    public function _unserialize(string $value): mixed {}

    /**
     * Return all redis master nodes
     *
     * @return array
     * @example
     * <pre>
     * $redisCluster->_masters(); // Will return [[0=>'127.0.0.1','6379'],[0=>'127.0.0.1','6380']]
     * </pre>
     */
    public function _masters(): array {}

    /**
     * Enter and exit transactional mode.
     *
     * @param int $value RedisCluster::MULTI|RedisCluster::PIPELINE
     *            Defaults to RedisCluster::MULTI.
     *            A RedisCluster::MULTI block of commands runs as a single transaction;
     *            a RedisCluster::PIPELINE block is simply transmitted faster to the server, but without any guarantee
     *            of atomicity. discard cancels a transaction.
     *
     * @return RedisCluster|bool returns the RedisCluster instance and enters multi-mode.
     * @throws RedisClusterException
     * Once in multi-mode, all subsequent method calls return the same object until exec() is called.
     * @link    https://redis.io/commands/multi
     * @example
     * <pre>
     * $ret = $redisCluster->multi()
     *      ->set('key1', 'val1')
     *      ->get('key1')
     *      ->set('key2', 'val2')
     *      ->get('key2')
     *      ->exec();
     *
     * //$ret == array (
     * //    0 => TRUE,
     * //    1 => 'val1',
     * //    2 => TRUE,
     * //    3 => 'val2');
     * </pre>
     */
    public function multi(int $value = Redis::MULTI): RedisCluster|bool {}

    /**
     * @return array|false
     * @throws RedisClusterException
     * @see    multi()
     * @link   https://redis.io/commands/exec
     */
    public function exec(): array|false {}

    /**
     * @see     multi()
     * @link    https://redis.io/commands/discard
     */
    public function discard(): bool {}

    /**
     * Watches a key for modifications by another client. If the key is modified between WATCH and EXEC,
     * the MULTI/EXEC transaction will fail (return FALSE). unwatch cancels all the watching of all keys by this client.
     *
     * @param string $key
     * @param string ...$other_keys
     *
     * @return RedisCluster|bool
     * @throws RedisClusterException
     * @link   https://redis.io/commands/watch
     * @example
     * <pre>
     * $redisCluster->watch('x');
     * // long code here during the execution of which other clients could well modify `x`
     * $ret = $redisCluster->multi()
     *          ->incr('x')
     *          ->exec();
     * // $ret = FALSE if x has been modified between the call to WATCH and the call to EXEC.
     * </pre>
     */
    public function watch(string $key, string ...$other_keys): RedisCluster|bool {}

    /**
     * @see     watch()
     * @link    https://redis.io/commands/unwatch
     * @throws  RedisClusterException
     */
    public function unwatch(): bool {}

    /**
     * Performs a synchronous save at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * If a save is already running, this command will fail and return FALSE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/save
     * @example
     * $redisCluster->save('x'); //key
     * $redisCluster->save(['127.0.0.1',6379]); //[host,port]
     */
    public function save(string|array $key_or_address): RedisCluster|bool {}

    /**
     * Performs a background save at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool    TRUE in case of success, FALSE in case of failure.
     * If a save is already running, this command will fail and return FALSE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/bgsave
     */
    public function bgSave(string|array $key_or_address): RedisCluster|bool {}

    /**
     * Removes all entries from the current database at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool Always TRUE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/flushdb
     */
    public function flushDB(string|array $key_or_address, bool $async = false): RedisCluster|bool {}

    /**
     * Removes all entries from all databases at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool Always TRUE.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/flushall
     */
    public function flushAll(string|array $key_or_address, bool $async = false): RedisCluster|bool {}

    /**
     * Returns the current database's size at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return RedisCluster|int DB size, in number of keys.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/dbsize
     * @example
     * <pre>
     * $count = $redisCluster->dbSize('x');
     * echo "Redis has $count keys\n";
     * </pre>
     */
    public function dbSize(string|array $key_or_address): RedisCluster|int {}

    /**
     * Starts the background rewrite of AOF (Append-Only File) at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool   TRUE in case of success, FALSE in case of failure.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/bgrewriteaof
     * @example $redisCluster->bgrewriteaof('x');
     */
    public function bgrewriteaof(string|array $key_or_address): RedisCluster|bool {}

    /**
     * Returns the timestamp of the last disk save at a specific node.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|int|false    timestamp.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/lastsave
     * @example $redisCluster->lastSave('x');
     */
    public function lastSave(string|array $key_or_address): RedisCluster|int|false {}

    /**
     * Returns an associative array of strings and integers
     *
     * @param string|array $key_or_address
     * @param string ...$sections SERVER | CLIENTS | MEMORY | PERSISTENCE | STATS | REPLICATION | CPU | CLASTER | KEYSPACE | COMANDSTATS
     *
     * Returns an associative array of strings and integers, with the following keys:
     * - redis_version
     * - redis_git_sha1
     * - redis_git_dirty
     * - redis_build_id
     * - redis_mode
     * - os
     * - arch_bits
     * - multiplexing_api
     * - atomicvar_api
     * - gcc_version
     * - process_id
     * - run_id
     * - tcp_port
     * - uptime_in_seconds
     * - uptime_in_days
     * - hz
     * - lru_clock
     * - executable
     * - config_file
     * - connected_clients
     * - client_longest_output_list
     * - client_biggest_input_buf
     * - blocked_clients
     * - used_memory
     * - used_memory_human
     * - used_memory_rss
     * - used_memory_rss_human
     * - used_memory_peak
     * - used_memory_peak_human
     * - used_memory_peak_perc
     * - used_memory_peak
     * - used_memory_overhead
     * - used_memory_startup
     * - used_memory_dataset
     * - used_memory_dataset_perc
     * - total_system_memory
     * - total_system_memory_human
     * - used_memory_lua
     * - used_memory_lua_human
     * - maxmemory
     * - maxmemory_human
     * - maxmemory_policy
     * - mem_fragmentation_ratio
     * - mem_allocator
     * - active_defrag_running
     * - lazyfree_pending_objects
     * - mem_fragmentation_ratio
     * - loading
     * - rdb_changes_since_last_save
     * - rdb_bgsave_in_progress
     * - rdb_last_save_time
     * - rdb_last_bgsave_status
     * - rdb_last_bgsave_time_sec
     * - rdb_current_bgsave_time_sec
     * - rdb_last_cow_size
     * - aof_enabled
     * - aof_rewrite_in_progress
     * - aof_rewrite_scheduled
     * - aof_last_rewrite_time_sec
     * - aof_current_rewrite_time_sec
     * - aof_last_bgrewrite_status
     * - aof_last_write_status
     * - aof_last_cow_size
     * - changes_since_last_save
     * - aof_current_size
     * - aof_base_size
     * - aof_pending_rewrite
     * - aof_buffer_length
     * - aof_rewrite_buffer_length
     * - aof_pending_bio_fsync
     * - aof_delayed_fsync
     * - loading_start_time
     * - loading_total_bytes
     * - loading_loaded_bytes
     * - loading_loaded_perc
     * - loading_eta_seconds
     * - total_connections_received
     * - total_commands_processed
     * - instantaneous_ops_per_sec
     * - total_net_input_bytes
     * - total_net_output_bytes
     * - instantaneous_input_kbps
     * - instantaneous_output_kbps
     * - rejected_connections
     * - maxclients
     * - sync_full
     * - sync_partial_ok
     * - sync_partial_err
     * - expired_keys
     * - evicted_keys
     * - keyspace_hits
     * - keyspace_misses
     * - pubsub_channels
     * - pubsub_patterns
     * - latest_fork_usec
     * - migrate_cached_sockets
     * - slave_expires_tracked_keys
     * - active_defrag_hits
     * - active_defrag_misses
     * - active_defrag_key_hits
     * - active_defrag_key_misses
     * - role
     * - master_replid
     * - master_replid2
     * - master_repl_offset
     * - second_repl_offset
     * - repl_backlog_active
     * - repl_backlog_size
     * - repl_backlog_first_byte_offset
     * - repl_backlog_histlen
     * - master_host
     * - master_port
     * - master_link_status
     * - master_last_io_seconds_ago
     * - master_sync_in_progress
     * - slave_repl_offset
     * - slave_priority
     * - slave_read_only
     * - master_sync_left_bytes
     * - master_sync_last_io_seconds_ago
     * - master_link_down_since_seconds
     * - connected_slaves
     * - min-slaves-to-write
     * - min-replicas-to-write
     * - min_slaves_good_slaves
     * - used_cpu_sys
     * - used_cpu_user
     * - used_cpu_sys_children
     * - used_cpu_user_children
     * - cluster_enabled
     *
     * @return  RedisCluster|array|false
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/info
     * @example
     * <pre>
     * $redisCluster->info();
     *
     * or
     *
     * $redisCluster->info("COMMANDSTATS"); //Information on the commands that have been run (>=2.6 only)
     * $redisCluster->info("CPU"); // just CPU information from Redis INFO
     * </pre>
     */
    public function info(string|array $key_or_address, string ...$sections): RedisCluster|array|false {}

    /**
     * @param string|array $key_or_address key or [host,port]
     *
     * @return mixed
     *   Returns the role of the instance in the context of replication
     * @throws RedisClusterException
     * @since  redis >= 2.8.12.
     * @link   https://redis.io/commands/role
     * @example
     * <pre>
     * $redisCluster->role(['127.0.0.1',6379]);
     * // [ 0=>'master',1 => 3129659, 2 => [ ['127.0.0.1','9001','3129242'], ['127.0.0.1','9002','3129543'] ] ]
     * </pre>
     */
    public function role(string|array $key_or_address): mixed {}

    /**
     * Returns a random key at the specified node
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return RedisCluster|bool|string an existing key in redis.
     * @throws RedisClusterException
     * @link    https://redis.io/commands/randomkey
     * @example
     * <pre>
     * $key = $redisCluster->randomKey('x');
     * $surprise = $redisCluster->get($key);  // who knows what's in there.
     * </pre>
     */
    public function randomKey(string|array $key_or_address): RedisCluster|bool|string {}

    /**
     * Return the specified node server time.
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  RedisCluster|bool|array If successfully, the time will come back as an associative array with element zero being the
     * unix timestamp, and element one being microseconds.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/time
     * @example
     * <pre>
     * var_dump( $redisCluster->time('x') );
     * //// Output:
     * //
     * // array(2) {
     * //   [0] => string(10) "1342364352"
     * //   [1] => string(6) "253002"
     * // }
     * </pre>
     */
    public function time(string|array $key_or_address): RedisCluster|bool|array {}

    /**
     * Check the specified node status
     *
     * @param string|array $key_or_address key or [host,port]
     *
     * @return  mixed STRING: +PONG on success. Throws a RedisClusterException object on connectivity error, as described
     *                 above.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/ping
     */
    public function ping(string|array $key_or_address, ?string $message = null): mixed {}

    /**
     * Returns message.
     *
     * @param string|array $key_or_address key or [host,port]
     * @param string        $msg
     *
     * @return mixed
     * @throws RedisClusterException
     */
    public function echo(string|array $key_or_address, string $msg): RedisCluster|string|false {}

    /**
     * Returns Array reply of details about all Redis Cluster commands.
     *
     * @return mixed array | bool
     * @throws RedisClusterException
     */
    public function command(mixed ...$extra_args): mixed {}

    /**
     * Send arbitrary things to the redis server at the specified node
     *
     * @param string|array $key_or_address    key or [host,port]
     * @param string       $command       Required command to send to the server.
     * @param mixed        ...$args       Optional variable amount of arguments to send to the server.
     *
     * @return  mixed
     * @throws  RedisClusterException
     */
    public function rawcommand(string|array $key_or_address, string $command, mixed ...$args): mixed {}

    /**
     * Executes cluster command
     *
     * @param string|array $key_or_address key or [host,port]
     * @param string       $command    Required command to send to the server.
     * @param mixed        $extra_args  Optional variable amount of arguments to send to the server.
     *
     * @return  mixed
     * @throws  RedisClusterException
     * @since redis >= 3.0
     *
     * @link  https://redis.io/commands#cluster
     * @example
     * <pre>
     * $redisCluster->cluster(['127.0.0.1',6379],'INFO');
     * </pre>
     */
    public function cluster(string|array $key_or_address, string $command, mixed ...$extra_args): mixed {}

    /**
     * Allows you to get information of the cluster client
     *
     * @param string|array $key_or_address key or [host,port]
     * @param string       $subcommand     can be: 'LIST', 'KILL', 'GETNAME', or 'SETNAME'
     * @param string|null  $arg            optional arguments
     *
     * @throws  RedisClusterException
     */
    public function client(string|array $key_or_address, string $subcommand, ?string $arg = null): array|string|bool {}

    /**
     * Get or Set the redis config keys.
     *
     * @param string|array $key_or_address key or [host,port]
     * @param string       $subcommand  either `GET` or `SET`
     * @param mixed        $extra_args
     *
     * @return  mixed   Associative array for `GET`, key -> value
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/config-get
     * @link    https://redis.io/commands/config-set
     * @example
     * <pre>
     * $redisCluster->config(['127.0.0.1',6379], "GET", "*max-*-entries*");
     * $redisCluster->config(['127.0.0.1',6379], "SET", "dir", "/var/run/redis/dumps/");
     * </pre>
     */
    public function config(string|array $key_or_address, string $subcommand, mixed ...$extra_args): mixed {}

    /**
     * A command allowing you to get information on the Redis pub/sub system.
     *
     * @param    string|array $key_or_address key or [host,port]
     *
     * @param    string       $keyword    String, which can be: "channels", "numsub", or "numpat"
     * @param    string ...$values   Optional, variant.
     *                                    For the "channels" subcommand, you can pass a string pattern.
     *                                    For "numsub" an array of channel names
     *
     * @return    mixed               Either an integer or an array.
     *                          - channels  Returns an array where the members are the matching channels.
     *                          - numsub    Returns a key/value array where the keys are channel names and
     *                                      values are their counts.
     *                          - numpat    Integer return containing the number active pattern subscriptions.
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/pubsub
     * @example
     * <pre>
     * $redisCluster->pubsub(['127.0.0.1',6379], 'channels'); // All channels
     * $redisCluster->pubsub(['127.0.0.1',6379], 'channels', '*pattern*'); // Just channels matching your pattern
     * $redisCluster->pubsub(['127.0.0.1',6379], 'numsub', array('chan1', 'chan2')); // Get subscriber counts for
     * 'chan1' and 'chan2'
     * $redisCluster->pubsub(['127.0.0.1',6379], 'numpat'); // Get the number of pattern subscribers
     * </pre>
     */
    public function pubsub(string|array $key_or_address, string ...$values): mixed {}

    /**
     * Execute the Redis SCRIPT command to perform various operations on the scripting subsystem.
     *
     * @param   string|array $key_or_address key or [host,port]
     * @param   mixed ...$args
     *
     * @return  mixed
     * @throws  RedisClusterException
     * @link    https://redis.io/commands/script-load
     * @link    https://redis.io/commands/script-kill
     * @link    https://redis.io/commands/script-flush
     * @link    https://redis.io/commands/script-exists
     * @example
     * <pre>
     * $redisCluster->script(['127.0.0.1',6379], 'load', $script);
     * $redisCluster->script(['127.0.0.1',6379], 'flush');
     * $redisCluster->script(['127.0.0.1',6379], 'kill');
     * $redisCluster->script(['127.0.0.1',6379], 'exists', $script1, [$script2, $script3, ...]);
     * </pre>
     *
     * SCRIPT LOAD will return the SHA1 hash of the passed script on success, and FALSE on failure.
     * SCRIPT FLUSH should always return TRUE
     * SCRIPT KILL will return true if a script was able to be killed and false if not
     * SCRIPT EXISTS will return an array with TRUE or FALSE for each passed script
     */
    public function script(string|array $key_or_address, mixed ...$args): mixed {}

    /**
     * This function is used in order to read and reset the Redis slow queries log.
     *
     * @param   string|array $key_or_address key or [host,port]
     * @param   mixed ...$args
     *
     * @throws  RedisClusterException
     * @link  https://redis.io/commands/slowlog
     * @example
     * <pre>
     * $redisCluster->slowlog(['127.0.0.1',6379],'get','2');
     * </pre>
     */
    public function slowlog(string|array $key_or_address, mixed ...$args): mixed {}

    /**
     * Add one or more geospatial items in the geospatial index represented using a sorted set
     *
     * @param string $key
     * @param float  $lng
     * @param float  $lat
     * @param string $member
     *
     * @throws  RedisClusterException
     * @link  https://redis.io/commands/geoadd
     * @example
     * <pre>
     * $redisCluster->geoadd('Sicily', 13.361389, 38.115556, 'Palermo'); // int(1)
     * $redisCluster->geoadd('Sicily', 15.087269, 37.502669, "Catania"); // int(1)
     * </pre>
     */
    public function geoadd(string $key, float $lng, float $lat, string $member, mixed ...$other_triples_and_options): RedisCluster|int|false {}

    /**
     * Returns members of a geospatial index as standard geohash strings
     *
     * @param string $key
     * @param string $member
     * @param string ...$other_members
     *
     * @throws  RedisClusterException
     * @example
     * <pre>
     * $redisCluster->geoAdd('Sicily', 13.361389, 38.115556, 'Palermo'); // int(1)
     * $redisCluster->geoAdd('Sicily', 15.087269, 37.502669, "Catania"); // int(1)
     * $redisCluster->geohash('Sicily','Palermo','Catania');//['sqc8b49rny0','sqdtr74hyu0']
     * </pre>
     */
    public function geohash(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    /**
     * Returns longitude and latitude of members of a geospatial index
     *
     * @param string $key
     * @param string $member
     * @param string ...$other_members
     *
     * @throws  RedisClusterException
     * @example
     * <pre>
     * $redisCluster->geoAdd('Sicily', 15.087269, 37.502669, "Catania"); // int(1)
     * $redisCluster->geopos('Sicily','Palermo');//[['13.36138933897018433','38.11555639549629859']]
     * </pre>
     */
    public function geopos(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    /**
     * Returns the distance between two members of a geospatial index
     *
     * @param    string $key
     * @param    string $src
     * @param    string $dest
     * @param    string|null $unit The unit must be one of the following, and defaults to meters:
     *                        m for meters.
     *                        km for kilometers.
     *                        mi for miles.
     *                        ft for feet.
     *
     * @throws  RedisClusterException
     * @link https://redis.io/commands/geoadd
     * @example
     * <pre>
     * $redisCluster->geoAdd('Sicily', 13.361389, 38.115556, 'Palermo'); // int(1)
     * $redisCluster->geoAdd('Sicily', 15.087269, 37.502669, "Catania"); // int(1)
     * $redisCluster->geodist('Sicily', 'Palermo' ,'Catania'); // float(166274.1516)
     * $redisCluster->geodist('Sicily', 'Palermo','Catania', 'km'); // float(166.2742)
     * </pre>
     */
    public function geodist(string $key, string $src, string $dest, ?string $unit = null): RedisCluster|float|false {}

    /**
     * Query a sorted set representing a geospatial index to fetch members matching a given maximum distance from a point
     *
     * @param    string $key
     * @param    float  $lng
     * @param    float  $lat
     * @param    float  $radius
     * @param    string $unit String can be: "m" for meters; "km" for kilometers , "mi" for miles, or "ft" for feet.
     * @param    array  $options
     *
     * @throws  RedisClusterException
     * @link  https://redis.io/commands/georadius
     * @example
     * <pre>
     * $redisCluster->del('Sicily');
     * $redisCluster->geoAdd('Sicily', 12.361389, 35.115556, 'Palermo'); // int(1)
     * $redisCluster->geoAdd('Sicily', 15.087269, 37.502669, "Catania"); // int(1)
     * $redisCluster->geoAdd('Sicily', 13.3585, 35.330022, "Agrigento"); // int(1)
     *
     * var_dump( $redisCluster->geoRadius('Sicily',13.3585, 35.330022, 300, 'km', ['WITHDIST' ,'DESC']) );
     *
     * array(3) {
     *    [0]=>
     *   array(2) {
     *        [0]=>
     *     string(7) "Catania"
     *        [1]=>
     *     string(8) "286.9362"
     *   }
     *   [1]=>
     *   array(2) {
     *        [0]=>
     *     string(7) "Palermo"
     *        [1]=>
     *     string(7) "93.6874"
     *   }
     *   [2]=>
     *   array(2) {
     *        [0]=>
     *     string(9) "Agrigento"
     *        [1]=>
     *     string(6) "0.0002"
     *   }
     * }
     * var_dump( $redisCluster->geoRadiusByMember('Sicily','Agrigento', 100, 'km', ['WITHDIST' ,'DESC']) );
     *
     * * array(2) {
     *    [0]=>
     *   array(2) {
     *        [0]=>
     *     string(7) "Palermo"
     *        [1]=>
     *     string(7) "93.6872"
     *   }
     *   [1]=>
     *   array(2) {
     *        [0]=>
     *     string(9) "Agrigento"
     *        [1]=>
     *     string(6) "0.0000"
     *   }
     * }
     *
     * <pre>
     */
    public function georadius(string $key, float $lng, float $lat, float $radius, string $unit, array $options = []): mixed {}

    /**
     * Query a sorted set representing a geospatial index to fetch members matching a given maximum distance from a member
     *
     * @param string $key
     * @param string $member
     * @param float  $radius
     * @param string $unit
     * @param array  $options
     *
     * @throws  RedisClusterException
     * @see geoRadius
     */
    public function georadiusbymember(string $key, string $member, float $radius, string $unit, array $options = []): mixed {}

    public function _compress(string $value): string {}

    public function _uncompress(string $value): string {}

    public function _pack(mixed $value): string {}

    public function _unpack(string $value): mixed {}

    public function _redir(): string|null {}

    public function acl(string|array $key_or_address, string $subcmd, string ...$args): mixed {}

    public function waitaof(string|array $key_or_address, int $numlocal, int $numreplicas, int $timeout): RedisCluster|array|false {}

    public function lMove(string $src, string $dst, string $wherefrom, string $whereto): Redis|string|false {}

    public function blmove(string $src, string $dst, string $wherefrom, string $whereto, float $timeout): Redis|string|false {}

    public function bzPopMax(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): array {}

    public function bzPopMin(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): array {}

    public function bzmpop(float $timeout, array $keys, string $from, int $count = 1): RedisCluster|array|null|false {}

    public function zmpop(array $keys, string $from, int $count = 1): RedisCluster|array|null|false {}

    public function blmpop(float $timeout, array $keys, string $from, int $count = 1): RedisCluster|array|null|false {}

    public function lmpop(array $keys, string $from, int $count = 1): RedisCluster|array|null|false {}

    public function copy(string $src, string $dst, ?array $options = null): RedisCluster|bool {}

    public function decrbyfloat(string $key, float $value): float {}

    public function eval(string $script, array $args = [], int $num_keys = 0): mixed {}

    public function eval_ro(string $script, array $args = [], int $num_keys = 0): mixed {}

    public function evalsha_ro(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function touch(mixed $key, mixed ...$other_keys): RedisCluster|int|bool {}

    public function expiretime(string $key): RedisCluster|int|false {}

    public function pexpiretime(string $key): RedisCluster|int|false {}

    public function georadius_ro(string $key, float $lng, float $lat, float $radius, string $unit, array $options = []): mixed {}

    public function georadiusbymember_ro(string $key, string $member, float $radius, string $unit, array $options = []): mixed {}

    public function geosearch(string $key, array|string $position, array|int|float $shape, string $unit, array $options = []): RedisCluster|array {}

    public function geosearchstore(string $dst, string $src, array|string $position, array|int|float $shape, string $unit, array $options = []): RedisCluster|array|int|false {}

    public function getDel(string $key): mixed {}

    public function getWithMeta(string $key): RedisCluster|array|false {}

    public function getEx(string $key, array $options = []): RedisCluster|string|false {}

    public function lcs(string $key1, string $key2, ?array $options = null): RedisCluster|string|array|int|false {}

    public function getTransferredBytes(): array|false {}

    public function clearTransferredBytes(): void {}

    public function expiremember(string $key, string $field, int $ttl, ?string $unit = null): Redis|int|false {}

    public function expirememberat(string $key, string $field, int $timestamp): Redis|int|false {}

    public function hRandField(string $key, ?array $options = null): RedisCluster|string|array {}

    public function hStrLen(string $key, string $field): RedisCluster|int|false {}

    public function hexpire(string $key, int $ttl, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hpexpire(string $key, int $ttl, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hexpireat(string $key, int $time, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hpexpireat(string $key, int $mstime, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function httl(string $key, array $fields): RedisCluster|array|false {}

    public function hpttl(string $key, array $fields): RedisCluster|array|false {}

    public function hexpiretime(string $key, array $fields): RedisCluster|array|false {}

    public function hpexpiretime(string $key, array $fields): RedisCluster|array|false {}

    public function hpersist(string $key, array $fields): RedisCluster|array|false {}

    public function lPos(string $key, mixed $value, ?array $options = null): Redis|null|bool|int|array {}

    public function sintercard(array $keys, int $limit = -1): RedisCluster|int|false {}

    public function sMisMember(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    public function sort_ro(string $key, ?array $options = null): RedisCluster|array|bool|int|string {}

    public function unlink(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    public function xack(string $key, string $group, array $ids): RedisCluster|int|false {}

    public function xadd(string $key, string $id, array $values, int $maxlen = 0, bool $approx = false): RedisCluster|string|false {}

    public function xclaim(string $key, string $group, string $consumer, int $min_iddle, array $ids, array $options): RedisCluster|string|array|false {}

    public function xdel(string $key, array $ids): RedisCluster|int|false {}

    public function xgroup(string $operation, ?string $key = null, ?string $group = null, ?string $id_or_consumer = null, bool $mkstream = false, int $entries_read = -2): mixed {}

    public function xautoclaim(string $key, string $group, string $consumer, int $min_idle, string $start, int $count = -1, bool $justid = false): RedisCluster|bool|array {}

    public function xinfo(string $operation, ?string $arg1 = null, ?string $arg2 = null, int $count = -1): mixed {}

    public function xlen(string $key): RedisCluster|int|false {}

    public function xpending(string $key, string $group, ?string $start = null, ?string $end = null, int $count = -1, ?string $consumer = null): RedisCluster|array|false {}

    public function xrange(string $key, string $start, string $end, int $count = -1): RedisCluster|bool|array {}

    public function xread(array $streams, int $count = -1, int $block = -1): RedisCluster|bool|array {}

    public function xreadgroup(string $group, string $consumer, array $streams, int $count = 1, int $block = 1): RedisCluster|bool|array {}

    public function xrevrange(string $key, string $start, string $end, int $count = -1): RedisCluster|bool|array {}

    public function xtrim(string $key, int $maxlen, bool $approx = false, bool $minid = false, int $limit = -1): RedisCluster|int|false {}

    public function zintercard(array $keys, int $limit = -1): RedisCluster|int|false {}

    public function zPopMax(string $key, ?int $value = null): RedisCluster|bool|array {}

    public function zPopMin(string $key, ?int $value = null): RedisCluster|bool|array {}

    public function zrangestore(string $dstkey, string $srckey, int $start, int $end, array|bool|null $options = null): RedisCluster|int|false {}

    public function zRandMember(string $key, ?array $options = null): RedisCluster|string|array {}

    public function zMscore(string $key, mixed $member, mixed ...$other_members): Redis|array|false {}

    public function zinter(array $keys, ?array $weights = null, ?array $options = null): RedisCluster|array|false {}

    public function zdiffstore(string $dst, array $keys): RedisCluster|int|false {}

    public function zunion(array $keys, ?array $weights = null, ?array $options = null): RedisCluster|array|false {}

    public function zdiff(array $keys, ?array $options = null): RedisCluster|array|false {}
}

class RedisClusterException extends Exception {}
