<?php
/**
 * Stubs for AMQP
 * https://pecl.php.net/package/amqp
 * https://github.com/php-amqp/php-amqp
 */

use JetBrains\PhpStorm\Deprecated;

/**
 * Passing in this constant as a flag will forcefully disable all other flags.
 * Use this if you want to temporarily disable the amqp.auto_ack ini setting.
 */
const AMQP_NOPARAM = 0;

/**
 * Passing in this constant as a flag to proper methods will forcefully ignore all other flags.
 * Do not send basic.consume request during AMQPQueue::consume(). Use this if you want to run callback on top of previously
 * declared consumers.
 */
const AMQP_JUST_CONSUME = 1;

/**
 * Durable exchanges and queues will survive a broker restart, complete with all of their data.
 */
const AMQP_DURABLE = 2;

/**
 * Passive exchanges and queues will not be redeclared, but the broker will throw an error if the exchange or queue does not exist.
 */
const AMQP_PASSIVE = 4;

/**
 * Valid for queues only, this flag indicates that only one client can be listening to and consuming from this queue.
 */
const AMQP_EXCLUSIVE = 8;

/**
 * For exchanges, the auto delete flag indicates that the exchange will be deleted as soon as no more queues are bound
 * to it. If no queues were ever bound the exchange, the exchange will never be deleted. For queues, the auto delete
 * flag indicates that the queue will be deleted as soon as there are no more listeners subscribed to it. If no
 * subscription has ever been active, the queue will never be deleted. Note: Exclusive queues will always be
 * automatically deleted with the client disconnects.
 */
const AMQP_AUTODELETE = 16;

/**
 * Clients are not allowed to make specific queue bindings to exchanges defined with this flag.
 */
const AMQP_INTERNAL = 32;

/**
 * When passed to the consume method for a clustered environment, do not consume from the local node.
 */
const AMQP_NOLOCAL = 64;

/**
 * When passed to the {@link AMQPQueue::get()} and {@link AMQPQueue::consume()} methods as a flag,
 * the messages will be immediately marked as acknowledged by the server upon delivery.
 */
const AMQP_AUTOACK = 128;

/**
 * Passed on queue creation, this flag indicates that the queue should be deleted if it becomes empty.
 */
const AMQP_IFEMPTY = 256;

/**
 * Passed on queue or exchange creation, this flag indicates that the queue or exchange should be
 * deleted when no clients are connected to the given queue or exchange.
 */
const AMQP_IFUNUSED = 512;

/**
 * When publishing a message, the message must be routed to a valid queue. If it is not, an error will be returned.
 */
const AMQP_MANDATORY = 1024;

/**
 * When publishing a message, mark this message for immediate processing by the broker. (High priority message.)
 */
const AMQP_IMMEDIATE = 2048;

/**
 * If set during a call to {@link AMQPQueue::ack()}, the delivery tag is treated as "up to and including", so that multiple
 * messages can be acknowledged with a single method. If set to zero, the delivery tag refers to a single message.
 * If the AMQP_MULTIPLE flag is set, and the delivery tag is zero, this indicates acknowledgement of all outstanding
 * messages.
 */
const AMQP_MULTIPLE = 4096;

/**
 * If set during a call to {@link AMQPExchange::bind()}, the server will not respond to the method.The client should not wait
 * for a reply method. If the server could not complete the method it will raise a channel or connection exception.
 */
const AMQP_NOWAIT = 8192;

/**
 * If set during a call to {@link AMQPQueue::nack()}, the message will be placed back to the queue.
 */
const AMQP_REQUEUE = 16384;

/**
 * A direct exchange type.
 */
const AMQP_EX_TYPE_DIRECT = 'direct';

/**
 * A fanout exchange type.
 */
const AMQP_EX_TYPE_FANOUT = 'fanout';

/**
 * A topic exchange type.
 */
const AMQP_EX_TYPE_TOPIC = 'topic';

/**
 * A header exchange type.
 */
const AMQP_EX_TYPE_HEADERS = 'headers';

const AMQP_OS_SOCKET_TIMEOUT_ERRNO = 536870947;

const PHP_AMQP_MAX_CHANNELS = 256;

const AMQP_SASL_METHOD_PLAIN = 0;

const AMQP_SASL_METHOD_EXTERNAL = 1;

/**
 * Default delivery mode, keeps the message in memory when the message is placed in a queue.
 */
const AMQP_DELIVERY_MODE_TRANSIENT = 1;

/**
 * Writes the message to the disk when the message is placed in a durable queue.
 */
const AMQP_DELIVERY_MODE_PERSISTENT = 2;

/**
 * Extension version string
 */
const AMQP_EXTENSION_VERSION = '1.1.12alpha3';

/**
 * Extension major version
 */
const AMQP_EXTENSION_VERSION_MAJOR = 0;

/**
 * Extension minor version
 */
const AMQP_EXTENSION_VERSION_MINOR = 1;

/**
 * Extension patch version
 */
const AMQP_EXTENSION_VERSION_PATCH = 12;

/**
 * Extension extra version suffix
 */
const AMQP_EXTENSION_VERSION_EXTRA = 'alpha3';

/**
 * Extension version ID
 */
const AMQP_EXTENSION_VERSION_ID = '10112';

/**
 * stub class representing AMQPBasicProperties from pecl-amqp
 */
class AMQPBasicProperties
{
    public function __construct(
        ?string $contentType = null,
        ?string $contentEncoding = null,
        array $headers = [],
        int $deliveryMode = AMQP_DELIVERY_MODE_TRANSIENT,
        int $priority = 0,
        ?string $correlationId = null,
        ?string $replyTo = null,
        ?string $expiration = null,
        ?string $messageId = null,
        ?int $timestamp = null,
        ?string $type = null,
        ?string $userId = null,
        ?string $appId = null,
        ?string $clusterId = null
    ) {}

    /**
     * Get the message content type.
     *
     * @return string|null The content type of the message.
     */
    public function getContentType() {}

    /**
     * Get the content encoding of the message.
     *
     * @return string|null The content encoding of the message.
     */
    public function getContentEncoding() {}

    /**
     * Get the headers of the message.
     *
     * @return array An array of key value pairs associated with the message.
     */
    public function getHeaders() {}

    /**
     * Get the delivery mode of the message.
     *
     * @return int The delivery mode of the message.
     */
    public function getDeliveryMode() {}

    /**
     * Get the priority of the message.
     *
     * @return int The message priority.
     */
    public function getPriority() {}

    /**
     * Get the message correlation id.
     *
     * @return string|null The correlation id of the message.
     */
    public function getCorrelationId() {}

    /**
     * Get the reply-to address of the message.
     *
     * @return string|null The contents of the reply to field.
     */
    public function getReplyTo() {}

    /**
     * Get the expiration of the message.
     *
     * @return string|null The message expiration.
     */
    public function getExpiration() {}

    /**
     * Get the message id of the message.
     *
     * @return string|null The message id
     */
    public function getMessageId() {}

    /**
     * Get the timestamp of the message.
     *
     * @return int|null The message timestamp.
     */
    public function getTimestamp() {}

    /**
     * Get the message type.
     *
     * @return string|null The message type.
     */
    public function getType() {}

    /**
     * Get the message user id.
     *
     * @return string|null The message user id.
     */
    public function getUserId() {}

    /**
     * Get the application id of the message.
     *
     * @return string|null The application id of the message.
     */
    public function getAppId() {}

    /**
     * Get the cluster id of the message.
     *
     * @return string|null The cluster id of the message.
     */
    public function getClusterId() {}
}

/**
 * stub class representing AMQPChannel from pecl-amqp
 */
class AMQPChannel
{
    /**
     * Commit a pending transaction.
     *
     * @throws AMQPChannelException    If no transaction was started prior to
     *                                 calling this method.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function commitTransaction() {}

    /**
     * Create an instance of an AMQPChannel object.
     *
     * @param AMQPConnection $amqp_connection An instance of AMQPConnection
     *                                        with an active connection to a
     *                                        broker.
     *
     * @throws AMQPConnectionException        If the connection to the broker
     *                                        was lost.
     */
    public function __construct(AMQPConnection $amqp_connection) {}

    /**
     * Check the channel connection.
     *
     * @return bool Indicates whether the channel is connected.
     */
    public function isConnected() {}

    /**
     * Closes the channel.
     *
     * @return void
     */
    public function close() {}

    /**
     * Return internal channel ID
     *
     * @return int
     */
    public function getChannelId() {}

    /**
     * Set the Quality Of Service settings for the given channel.
     *
     * Specify the amount of data to prefetch in terms of window size (octets)
     * or number of messages from a queue during a AMQPQueue::consume() or
     * AMQPQueue::get() method call. The client will prefetch data up to size
     * octets or count messages from the server, whichever limit is hit first.
     * Setting either value to 0 will instruct the client to ignore that
     * particular setting. A call to AMQPChannel::qos() will overwrite any
     * values set by calling AMQPChannel::setPrefetchSize() and
     * AMQPChannel::setPrefetchCount(). If the call to either
     * AMQPQueue::consume() or AMQPQueue::get() is done with the AMQP_AUTOACK
     * flag set, the client will not do any prefetching of data, regardless of
     * the QOS settings.
     *
     * @param int $size   The window size, in octets, to prefetch.
     * @param int $count  The number of messages to prefetch.
     * @param bool    $global TRUE for global, FALSE for consumer. FALSE by default.
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function qos($size, $count, $global = false) {}

    /**
     * Rollback a transaction.
     *
     * Rollback an existing transaction. AMQPChannel::startTransaction() must
     * be called prior to this.
     *
     * @throws AMQPChannelException    If no transaction was started prior to
     *                                 calling this method.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function rollbackTransaction() {}

    /**
     * Set the number of messages to prefetch from the broker for each consumer.
     *
     * Set the number of messages to prefetch from the broker during a call to
     * AMQPQueue::consume() or AMQPQueue::get().
     *
     * @param int $count The number of messages to prefetch.
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function setPrefetchCount($count) {}

    /**
     * Get the number of messages to prefetch from the broker for each consumer.
     *
     * @return int
     */
    public function getPrefetchCount() {}

    /**
     * Set the window size to prefetch from the broker for each consumer.
     *
     * Set the prefetch window size, in octets, during a call to
     * AMQPQueue::consume() or AMQPQueue::get(). Any call to this method will
     * automatically set the prefetch message count to 0, meaning that the
     * prefetch message count setting will be ignored. If the call to either
     * AMQPQueue::consume() or AMQPQueue::get() is done with the AMQP_AUTOACK
     * flag set, this setting will be ignored.
     *
     * @param int $size The window size, in octets, to prefetch.
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function setPrefetchSize($size) {}

    /**
     * Get the window size to prefetch from the broker for each consumer.
     *
     * @return int
     */
    public function getPrefetchSize() {}

    /**
     * Set the number of messages to prefetch from the broker across all consumers.
     *
     * Set the number of messages to prefetch from the broker during a call to
     * AMQPQueue::consume() or AMQPQueue::get().
     *
     * @param int $count The number of messages to prefetch.
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function setGlobalPrefetchCount($count) {}

    /**
     * Get the number of messages to prefetch from the broker across all consumers.
     *
     * @return int
     */
    public function getGlobalPrefetchCount() {}

    /**
     * Set the window size to prefetch from the broker for all consumers.
     *
     * Set the prefetch window size, in octets, during a call to
     * AMQPQueue::consume() or AMQPQueue::get(). Any call to this method will
     * automatically set the prefetch message count to 0, meaning that the
     * prefetch message count setting will be ignored. If the call to either
     * AMQPQueue::consume() or AMQPQueue::get() is done with the AMQP_AUTOACK
     * flag set, this setting will be ignored.
     *
     * @param int $size The window size, in octets, to prefetch.
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function setGlobalPrefetchSize($size) {}

    /**
     * Get the window size to prefetch from the broker for all consumers.
     *
     * @return int
     */
    public function getGlobalPrefetchSize() {}

    /**
     * Start a transaction.
     *
     * This method must be called on the given channel prior to calling
     * AMQPChannel::commitTransaction() or AMQPChannel::rollbackTransaction().
     *
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function startTransaction() {}

    /**
     * Get the AMQPConnection object in use
     *
     * @return AMQPConnection
     */
    public function getConnection() {}

    /**
     * Redeliver unacknowledged messages.
     *
     * @param bool $requeue
     *
     * @return void
     */
    public function basicRecover($requeue = true) {}

    /**
     * Set the channel to use publisher acknowledgements. This can only used on a non-transactional channel.
     *
     * @return void
     */
    public function confirmSelect() {}

    /**
     * Set callback to process basic.ack and basic.nac AMQP server methods (applicable when channel in confirm mode).
     *
     * @param callable|null $ack_callback
     * @param callable|null $nack_callback
     *
     * Callback functions with all arguments have the following signature:
     *
     *      function ack_callback(int $deliveryTag, bool $multiple) : bool;
     *      function nack_callback(int $deliveryTag, bool $multiple, bool $requeue) : bool;
     *
     * and should return boolean false when wait loop should be canceled.
     *
     * Note, basic.nack server method will only be delivered if an internal error occurs in the Erlang process
     * responsible for a queue (see https://www.rabbitmq.com/confirms.html for details).
     *
     * @return void
     */
    public function setConfirmCallback(callable $ack_callback = null, callable $nack_callback = null) {}

    /**
     * Wait until all messages published since the last call have been either ack'd or nack'd by the broker.
     *
     * Note, this method also catch all basic.return message from server.
     *
     * @param float $timeout Timeout in seconds. May be fractional.
     *
     * @throws AMQPQueueException If timeout occurs.
     *
     * @return void
     */
    public function waitForConfirm($timeout = 0.0) {}

    /**
     * Set callback to process basic.return AMQP server method
     *
     * @param callable|null $return_callback
     *
     * Callback function with all arguments has the following signature:
     *
     *      function callback(int $reply_code,
     *                        string $reply_text,
     *                        string $exchange,
     *                        string $routingKey,
     *                        AMQPBasicProperties $properties,
     *                        string $body) : bool;
     *
     * and should return boolean false when wait loop should be canceled.
     *
     * @return void
     */
    public function setReturnCallback(callable $return_callback = null) {}

    /**
     * Start wait loop for basic.return AMQP server methods
     *
     * @param float $timeout Timeout in seconds. May be fractional.
     *
     * @throws AMQPQueueException If timeout occurs.
     *
     * @return void
     */
    public function waitForBasicReturn($timeout = 0.0) {}

    /**
     * Return array of current consumers where key is consumer and value is AMQPQueue consumer is running on
     *
     * @return AMQPQueue[]
     */
    public function getConsumers() {}
}

/**
 * stub class representing AMQPChannelException from pecl-amqp
 */
class AMQPChannelException extends AMQPException {}

/**
 * stub class representing AMQPConnection from pecl-amqp
 */
class AMQPConnection
{
    /**
     * Create an instance of AMQPConnection.
     *
     * Creates an AMQPConnection instance representing a connection to an AMQP
     * broker. A connection will not be established until
     * AMQPConnection::connect() is called.
     *
     *  $credentials = array(
     *      'host'  => amqp.host The host to connect too. Note: Max 1024 characters.
     *      'port'  => amqp.port Port on the host.
     *      'vhost' => amqp.vhost The virtual host on the host. Note: Max 128 characters.
     *      'login' => amqp.login The login name to use. Note: Max 128 characters.
     *      'password' => amqp.password Password. Note: Max 128 characters.
     *      'read_timeout'  => Timeout in for income activity. Note: 0 or greater seconds. May be fractional.
     *      'write_timeout' => Timeout in for outcome activity. Note: 0 or greater seconds. May be fractional.
     *      'connect_timeout' => Connection timeout. Note: 0 or greater seconds. May be fractional.
     *      'rpc_timeout' => RPC timeout. Note: 0 or greater seconds. May be fractional.
     *
     *      Connection tuning options (see http://www.rabbitmq.com/amqp-0-9-1-reference.html#connection.tune for details):
     *      'channel_max' => Specifies highest channel number that the server permits. 0 means standard extension limit
     *                       (see PHP_AMQP_MAX_CHANNELS constant)
     *      'frame_max'   => The largest frame size that the server proposes for the connection, including frame header
     *                       and end-byte. 0 means standard extension limit (depends on librabbimq default frame size limit)
     *      'heartbeat'   => The delay, in seconds, of the connection heartbeat that the server wants.
     *                       0 means the server does not want a heartbeat. Note, librabbitmq has limited heartbeat support,
     *                       which means heartbeats checked only during blocking calls.
     *
     *      TLS support (see https://www.rabbitmq.com/ssl.html for details):
     *      'cacert' => Path to the CA cert file in PEM format..
     *      'cert'   => Path to the client certificate in PEM foramt.
     *      'key'    => Path to the client key in PEM format.
     *      'verify' => Enable or disable peer verification. If peer verification is enabled then the common name in the
     *                  server certificate must match the server name. Peer verification is enabled by default.
     *
     *      'connection_name' => A user determined name for the connection
     * )
     *
     * @param array $credentials Optional array of credential information for
     *                           connecting to the AMQP broker.
     */
    public function __construct(array $credentials = []) {}

    /**
     * Check whether the connection to the AMQP broker is still valid.
     *
     * Cannot reliably detect dropped connections or unusual socket errors, as it does not actively
     * engage the socket.
     *
     * @return bool TRUE if connected, FALSE otherwise.
     */
    public function isConnected() {}

    /**
     * Whether connection persistent.
     *
     * When no connection is established, it will always return FALSE. The same disclaimer as for
     * {@see AMQPConnection::isConnected()} applies.
     *
     * @return bool TRUE if persistently connected, FALSE otherwise.
     */
    public function isPersistent() {}

    /**
     * Establish a transient connection with the AMQP broker.
     *
     * This method will initiate a connection with the AMQP broker.
     *
     * @throws AMQPConnectionException
     * @return void
     */
    public function connect() {}

    /**
     * Closes the transient connection with the AMQP broker.
     *
     * This method will close an open connection with the AMQP broker.
     *
     * @throws AMQPConnectionException When attempting to disconnect a persistent connection
     *
     * @return void
     */
    public function disconnect() {}

    /**
     * Close any open transient connections and initiate a new one with the AMQP broker.
     *
     * @return void
     */
    public function reconnect() {}

    /**
     * Establish a persistent connection with the AMQP broker.
     *
     * This method will initiate a connection with the AMQP broker
     * or reuse an existing one if present.
     *
     * @throws AMQPConnectionException
     * @return void
     */
    public function pconnect() {}

    /**
     * Closes a persistent connection with the AMQP broker.
     *
     * This method will close an open persistent connection with the AMQP
     * broker.
     *
     * @throws AMQPConnectionException When attempting to disconnect a transient connection
     *
     * @return void
     */
    public function pdisconnect() {}

    /**
     * Close any open persistent connections and initiate a new one with the AMQP broker.
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function preconnect() {}

    /**
     * Get the configured host.
     *
     * @return string The configured hostname of the broker
     */
    public function getHost() {}

    /**
     * Get the configured login.
     *
     * @return string The configured login as a string.
     */
    public function getLogin() {}

    /**
     * Get the configured password.
     *
     * @return string The configured password as a string.
     */
    public function getPassword() {}

    /**
     * Get the configured port.
     *
     * @return int The configured port as an integer.
     */
    public function getPort() {}

    /**
     * Get the configured vhost.
     *
     * @return string The configured virtual host as a string.
     */
    public function getVhost() {}

    /**
     * Set the hostname used to connect to the AMQP broker.
     *
     * @param string $host The hostname of the AMQP broker.
     *
     * @throws AMQPConnectionException If host is longer then 1024 characters.
     *
     * @return void
     */
    public function setHost($host) {}

    /**
     * Set the login string used to connect to the AMQP broker.
     *
     * @param string $login The login string used to authenticate
     *                      with the AMQP broker.
     *
     * @throws AMQPConnectionException If login is longer then 32 characters.
     *
     * @return void
     */
    public function setLogin($login) {}

    /**
     * Set the password string used to connect to the AMQP broker.
     *
     * @param string $password The password string used to authenticate
     *                         with the AMQP broker.
     *
     * @throws AMQPConnectionException If password is longer then 32characters.
     *
     * @return void
     */
    public function setPassword($password) {}

    /**
     * Set the port used to connect to the AMQP broker.
     *
     * @param int $port The port used to connect to the AMQP broker.
     *
     * @throws AMQPConnectionException If port is longer not between
     *                                 1 and 65535.
     *
     * @return void
     */
    public function setPort($port) {}

    /**
     * Sets the virtual host to which to connect on the AMQP broker.
     *
     * @param string $vhost The virtual host to use on the AMQP
     *                      broker.
     *
     * @throws AMQPConnectionException If host is longer then 32 characters.
     *
     * @return void
     */
    public function setVhost($vhost) {}

    /**
     * Sets the interval of time to wait for income activity from AMQP broker
     *
     * @deprecated use AMQPConnection::setReadTimeout($timeout) instead
     *
     * @param float $timeout
     *
     * @throws AMQPConnectionException If timeout is less than 0.
     *
     * @return void
     */
    #[Deprecated(replacement: "%class%->setReadTimout(%parameter0%)")]
    public function setTimeout($timeout) {}

    /**
     * Get the configured interval of time to wait for income activity
     * from AMQP broker
     *
     * @deprecated use AMQPConnection::getReadTimeout() instead
     *
     * @return float
     */
    #[Deprecated(replacement: '%class%->getReadTimout(%parameter0%)')]
    public function getTimeout() {}

    /**
     * Sets the interval of time to wait for income activity from AMQP broker
     *
     * @param float $timeout
     *
     * @throws AMQPConnectionException If timeout is less than 0.
     *
     * @return void
     */
    public function setReadTimeout($timeout) {}

    /**
     * Get the configured interval of time to wait for income activity
     * from AMQP broker
     *
     * @return float
     */
    public function getReadTimeout() {}

    /**
     * Sets the interval of time to wait for outcome activity to AMQP broker
     *
     * @param float $timeout
     *
     * @throws AMQPConnectionException If timeout is less than 0.
     *
     * @return void
     */
    public function setWriteTimeout($timeout) {}

    /**
     * Get the configured interval of time to wait for outcome activity
     * to AMQP broker
     *
     * @return float
     */
    public function getWriteTimeout() {}

    /**
     * Get the configured timeout (in seconds) for connecting to the AMQP broker
     */
    public function getConnectTimeout(): float {}

    /**
     * Sets the interval of time to wait for RPC activity to AMQP broker
     *
     * @param float $timeout
     *
     * @throws AMQPConnectionException If timeout is less than 0.
     *
     * @return void
     */
    public function setRpcTimeout($timeout) {}

    /**
     * Get the configured interval of time to wait for RPC activity
     * to AMQP broker
     *
     * @return float
     */
    public function getRpcTimeout() {}

    /**
     * Return last used channel id during current connection session.
     *
     * @return int
     */
    public function getUsedChannels() {}

    /**
     * Get the maximum number of channels the connection can handle.
     *
     * When connection is connected, effective connection value returned, which is normally the same as original
     * correspondent value passed to constructor, otherwise original value passed to constructor returned.
     *
     * @return int
     */
    public function getMaxChannels() {}

    /**
     * Get max supported frame size per connection in bytes.
     *
     * When connection is connected, effective connection value returned, which is normally the same as original
     * correspondent value passed to constructor, otherwise original value passed to constructor returned.
     *
     * @return int
     */
    public function getMaxFrameSize() {}

    /**
     * Get number of seconds between heartbeats of the connection in seconds.
     *
     * When connection is connected, effective connection value returned, which is normally the same as original
     * correspondent value passed to constructor, otherwise original value passed to constructor returned.
     *
     * @return int
     */
    public function getHeartbeatInterval() {}

    /**
     * Get path to the CA cert file in PEM format
     *
     * @return string|null
     */
    public function getCACert() {}

    /**
     * Set path to the CA cert file in PEM format
     *
     * @param string $cacert
     *
     * @return void
     */
    public function setCACert($cacert) {}

    /**
     * Get path to the client certificate in PEM format
     *
     * @return string|null
     */
    public function getCert() {}

    /**
     * Set path to the client certificate in PEM format
     *
     * @param string $cert
     *
     * @return void
     */
    public function setCert($cert) {}

    /**
     * Get path to the client key in PEM format
     *
     * @return string|null
     */
    public function getKey() {}

    /**
     * Set path to the client key in PEM format
     *
     * @param string|null $key
     *
     * @return void
     */
    public function setKey($key) {}

    /**
     * Get whether peer verification enabled or disabled
     *
     * @return bool
     */
    public function getVerify() {}

    /**
     * Enable or disable peer verification
     *
     * @param bool $verify
     *
     * @return void
     */
    public function setVerify($verify) {}

    /**
     * set authentication method
     *
     * @param int $saslMethod AMQP_SASL_METHOD_PLAIN | AMQP_SASL_METHOD_EXTERNAL
     *
     * @return void
     */
    public function setSaslMethod($method) {}

    /**
     * Get authentication mechanism configuration
     *
     * @return int AMQP_SASL_METHOD_PLAIN | AMQP_SASL_METHOD_EXTERNAL
     */
    public function getSaslMethod() {}

    public function setConnectionName(?string $connectionName): void {}

    public function getConnectionName(): ?string {}
}

/**
 * stub class representing AMQPConnectionException from pecl-amqp
 */
class AMQPConnectionException extends AMQPException {}

/**
 * Interface representing AMQP values
 */
interface AMQPValue
{
    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null;
}

/**
 * stub class representing AMQPDecimal from pecl-amqp
 */
final /* readonly */ class AMQPDecimal implements AMQPValue
{
    public const EXPONENT_MIN = 0;
    public const EXPONENT_MAX = 255;
    public const SIGNIFICAND_MIN = 0;
    public const SIGNIFICAND_MAX = 4294967295;

    /**
     * @param $exponent
     * @param $significand
     *
     * @throws AMQPExchangeValue
     */
    public function __construct($exponent, $significand) {}

    /** @return int */
    public function getExponent() {}

    /** @return int */
    public function getSignificand() {}

    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null {}
}

/**
 * stub class representing AMQPEnvelope from pecl-amqp
 */
class AMQPEnvelope extends AMQPBasicProperties
{
    /**
     * Get the body of the message.
     *
     * @return string The contents of the message body.
     */
    public function getBody() {}

    /**
     * Get the routing key of the message.
     *
     * @return string The message routing key.
     */
    public function getRoutingKey() {}

    /**
     * Get the consumer tag of the message.
     *
     * @return string|null The consumer tag of the message.
     */
    public function getConsumerTag() {}

    /**
     * Get the delivery tag of the message.
     *
     * @return int|null The delivery tag of the message.
     */
    public function getDeliveryTag() {}

    /**
     * Get the exchange name on which the message was published.
     *
     * @return string|null The exchange name on which the message was published.
     */
    public function getExchangeName() {}

    /**
     * Whether this is a redelivery of the message.
     *
     * Whether this is a redelivery of a message. If this message has been
     * delivered and AMQPEnvelope::nack() was called, the message will be put
     * back on the queue to be redelivered, at which point the message will
     * always return TRUE when this method is called.
     *
     * @return bool TRUE if this is a redelivery, FALSE otherwise.
     */
    public function isRedelivery() {}

    /**
     * Get a specific message header.
     *
     * @param string $headerName Name of the header to get the value from.
     *
     * @return mixed The contents of the specified header or null if not set.
     */
    public function getHeader($headerName) {}

    /**
     * Check whether specific message header exists.
     *
     * @param string $headerName Name of the header to check.
     *
     * @return bool
     */
    public function hasHeader($headerName) {}
}

/**
 * stub class representing AMQPEnvelopeException from pecl-amqp
 */
class AMQPEnvelopeException extends AMQPException
{
    public function getEnvelope(): AMQPEnvelope {}
}

/**
 * stub class representing AMQPException from pecl-amqp
 */
class AMQPException extends Exception {}

/**
 * stub class representing AMQPExchange from pecl-amqp
 */
class AMQPExchange
{
    /**
     * Create an instance of AMQPExchange.
     *
     * Returns a new instance of an AMQPExchange object, associated with the
     * given AMQPChannel object.
     *
     * @param AMQPChannel $channel A valid AMQPChannel object, connected
     *                                  to a broker.
     *
     * @throws AMQPExchangeException   When amqp_channel is not connected to
     *                                 a broker.
     * @throws AMQPConnectionException If the connection to the broker was
     *                                 lost.
     */
    public function __construct(AMQPChannel $channel) {}

    /**
     * Bind to another exchange.
     *
     * Bind an exchange to another exchange using the specified routing key.
     *
     * @param string $exchangeName Name of the exchange to bind.
     * @param string $routingKey   The routing key to use for binding.
     * @param array  $arguments     Additional binding arguments.
     *
     * @throws AMQPExchangeException   On failure.
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function bind($exchangeName, $routingKey = '', array $arguments = []) {}

    /**
     * Remove binding to another exchange.
     *
     * Remove a routing key binding on an another exchange from the given exchange.
     *
     * @param string $exchangeName Name of the exchange to bind.
     * @param string $routingKey   The routing key to use for binding.
     * @param array  $arguments     Additional binding arguments.
     *
     * @throws AMQPExchangeException   On failure.
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function unbind($exchangeName, $routingKey = '', array $arguments = []) {}

    /**
     * Declare a new exchange on the broker.
     *
     * @throws AMQPExchangeException   On failure.
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function declareExchange() {}

    /**
     * Declare a new exchange on the broker.
     *
     * @throws AMQPExchangeException On failure.
     * @throws AMQPChannelException If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function declare(): void {}

    /**
     * Delete the exchange from the broker.
     *
     * @param string  $exchangeName Optional name of exchange to delete.
     * @param int $flags        Optionally AMQP_IFUNUSED can be specified
     *                              to indicate the exchange should not be
     *                              deleted until no clients are connected to
     *                              it.
     *
     * @throws AMQPExchangeException   On failure.
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function delete($exchangeName = null, $flags = AMQP_NOPARAM) {}

    /**
     * Get the argument associated with the given key.
     * Get the argument associated with the given key.
     *
     * @param string $argumentName The key to look up.
     *
     * @throws AMQPExchangeException If key does not exist
     *
     * @return bool|int|float|string|null
     */
    public function getArgument($argumentName) {}

    /**
     * Check whether argument associated with the given key exists.
     *
     * @param string $argumentName The key to look up.
     *
     * @return bool
     */
    public function hasArgument($argumentName) {}

    /**
     * Get all arguments set on the given exchange.
     *
     * @return array An array containing all of the set key/value pairs.
     */
    public function getArguments() {}

    /**
     * Get all the flags currently set on the given exchange.
     *
     * @return int An integer bitmask of all the flags currently set on this
     *             exchange object.
     */
    public function getFlags() {}

    /**
     * Get the configured name.
     *
     * @return string|null The configured name as a string.
     */
    public function getName() {}

    /**
     * Get the configured type.
     *
     * @return string|null The configured type as a string.
     */
    public function getType() {}

    /**
     * Publish a message to an exchange.
     *
     * Publish a message to the exchange represented by the AMQPExchange object.
     *
     * @param string  $message     The message to publish.
     * @param string|null $routingKey The optional routing key to which to
     *                             publish to.
     * @param int|null $flags       One or more of AMQP_MANDATORY and
     *                             AMQP_IMMEDIATE.
     * @param array   $headers  One of content_type, content_encoding,
     *                             message_id, user_id, app_id, delivery_mode,
     *                             priority, timestamp, expiration, type
     *                             or reply_to, headers.
     *
     * @throws AMQPExchangeException   On failure.
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function publish(
        $message,
        $routingKey = null,
        $flags = null,
        array $headers = []
    ) {}

    /**
     * Set the value for the given key.
     *
     * @param string         $argumentName   Name of the argument to set.
     * @param string|int $argumentValue Value of the argument to set.
     *
     * @return void
     */
    public function setArgument($argumentName, $argumentValue) {}

    /**
     * Set the value for the given key.
     *
     * @param string $argumentName Name of the argument to remove.
     */
    public function removeArgument(string $argumentName): void {}

    /**
     * Set all arguments on the exchange.
     *
     * @param array $arguments An array of key/value pairs of arguments.
     *
     * @return bool TRUE on success or FALSE on failure.
     */
    public function setArguments(array $arguments) {}

    /**
     * Set the flags on an exchange.
     *
     * @param int|null $flags A bitmask of flags. This call currently only
     *                       considers the following flags:
     *                       AMQP_DURABLE, AMQP_PASSIVE
     *                       (and AMQP_DURABLE, if librabbitmq version >= 0.5.3)
     *
     * @return void
     */
    public function setFlags($flags) {}

    /**
     * Set the name of the exchange.
     *
     * @param string $exchangeName The name of the exchange to set as string.
     *
     * @return void
     */
    public function setName($exchangeName) {}

    /**
     * Set the type of the exchange.
     *
     * Set the type of the exchange. This can be any of AMQP_EX_TYPE_DIRECT,
     * AMQP_EX_TYPE_FANOUT, AMQP_EX_TYPE_HEADERS or AMQP_EX_TYPE_TOPIC.
     *
     * @param string $exchangeType The type of exchange as a string.
     *
     * @return void
     */
    public function setType($exchangeType) {}

    /**
     * Get the AMQPChannel object in use
     *
     * @return AMQPChannel
     */
    public function getChannel() {}

    /**
     * Get the AMQPConnection object in use
     *
     * @return AMQPConnection
     */
    public function getConnection() {}
}

/**
 * stub class representing AMQPExchangeException from pecl-amqp
 */
class AMQPExchangeException extends AMQPException {}

/**
 * stub class representing AMQPQueue from pecl-amqp
 */
class AMQPQueue
{
    /**
     * Create an instance of an AMQPQueue object.
     *
     * @param AMQPChannel $channel The amqp channel to use.
     *
     * @throws AMQPQueueException      When amqp_channel is not connected to a
     *                                 broker.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     */
    public function __construct(AMQPChannel $channel) {}

    /**
     * Acknowledge the receipt of a message.
     *
     * This method allows the acknowledgement of a message that is retrieved
     * without the AMQP_AUTOACK flag through AMQPQueue::get() or
     * AMQPQueue::consume()
     *
     * @param int $deliveryTag The message delivery tag of which to
     *                              acknowledge receipt.
     * @param int|null $flags        The only valid flag that can be passed is
     *                              AMQP_MULTIPLE.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return void
     */
    public function ack($deliveryTag, $flags = null) {}

    /**
     * Bind the given queue to a routing key on an exchange.
     *
     * @param string $exchangeName Name of the exchange to bind to.
     * @param string|null $routingKey   Pattern or routing key to bind with.
     * @param array  $arguments     Additional binding arguments.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return bool
     */
    public function bind($exchangeName, $routingKey = null, array $arguments = []) {}

    /**
     * Cancel a queue that is already bound to an exchange and routing key.
     *
     * @param string $consumer_tag The consumer tag cancel. If no tag provided,
     *                             or it is empty string, the latest consumer
     *                             tag on this queue will be used and after
     *                             successful request it will set to null.
     *                             If it also empty, no `basic.cancel`
     *                             request will be sent. When consumer_tag give
     *                             and it equals to latest consumer_tag on queue,
     *                             it will be interpreted as latest consumer_tag usage.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return bool
     */
    public function cancel($consumer_tag = '') {}

    /**
     * Consume messages from a queue.
     *
     * Blocking function that will retrieve the next message from the queue as
     * it becomes available and will pass it off to the callback.
     *
     * @param callable|null $callback    A callback function to which the
     *                              consumed message will be passed. The
     *                              function must accept at a minimum
     *                              one parameter, an AMQPEnvelope object,
     *                              and an optional second parameter
     *                              the AMQPQueue object from which callback
     *                              was invoked. The AMQPQueue::consume() will
     *                              not return the processing thread back to
     *                              the PHP script until the callback
     *                              function returns FALSE.
     *                              If the callback is omitted or null is passed,
     *                              then the messages delivered to this client will
     *                              be made available to the first real callback
     *                              registered. That allows one to have a single
     *                              callback consuming from multiple queues.
     * @param int|null $flags        A bitmask of any of the flags: AMQP_AUTOACK,
     *                              AMQP_JUST_CONSUME. Note: when AMQP_JUST_CONSUME
     *                              flag used all other flags are ignored and
     *                              $consumerTag parameter has no sense.
     *                              AMQP_JUST_CONSUME flag prevent from sending
     *                              `basic.consume` request and just run $callback
     *                              if it provided. Calling method with empty $callback
     *                              and AMQP_JUST_CONSUME makes no sense.
     * @param string|null   $consumerTag A string describing this consumer. Used
     *                              for canceling subscriptions with cancel().
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     * @throws AMQPEnvelopeException   When no queue found for envelope.
     * @throws AMQPQueueException      If timeout occurs or queue is not exists.
     *
     * @return void
     */
    public function consume(
        callable $callback = null,
        $flags = null,
        $consumerTag = null
    ) {}

    /**
     * Declare a new queue on the broker.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     * @throws AMQPQueueException      On failure.
     *
     * @return int the message count.
     */
    public function declareQueue() {}

    /**
     * Declare a new queue on the broker.
     *
     * @throws AMQPChannelException If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     * @throws AMQPQueueException On failure.
     *
     * @return int the message count.
     */
    public function declare(): int {}

    /**
     * Delete a queue from the broker.
     *
     * This includes its entire contents of unread or unacknowledged messages.
     *
     * @param int $flags        Optionally AMQP_IFUNUSED can be specified
     *                              to indicate the queue should not be
     *                              deleted until no clients are connected to
     *                              it.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return int The number of deleted messages.
     */
    public function delete($flags = AMQP_NOPARAM) {}

    /**
     * Retrieve the next message from the queue.
     *
     * Retrieve the next available message from the queue. If no messages are
     * present in the queue, this function will return NULL immediately. This
     * is a non blocking alternative to the AMQPQueue::consume() method.
     * Currently, the only supported flag for the flags parameter is
     * AMQP_AUTOACK. If this flag is passed in, then the message returned will
     * automatically be marked as acknowledged by the broker as soon as the
     * frames are sent to the client.
     *
     * @param int $flags A bitmask of supported flags for the
     *                       method call. Currently, the only the
     *                       supported flag is AMQP_AUTOACK. If this
     *                       value is not provided, it will use the
     *                       value of ini-setting amqp.auto_ack.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     * @throws AMQPQueueException      If queue is not exist.
     *
     * @return AMQPEnvelope|null
     */
    public function get($flags = null) {}

    /**
     * Get all the flags currently set on the given queue.
     *
     * @return int An integer bitmask of all the flags currently set on this
     *             exchange object.
     */
    public function getFlags(): int {}

    /**
     * Get the configured name.
     *
     * @return string|null The configured name as a string.
     */
    public function getName(): ?string {}

    /**
     * Mark a message as explicitly not acknowledged.
     *
     * Mark the message identified by delivery_tag as explicitly not
     * acknowledged. This method can only be called on messages that have not
     * yet been acknowledged, meaning that messages retrieved with by
     * AMQPQueue::consume() and AMQPQueue::get() and using the AMQP_AUTOACK
     * flag are not eligible. When called, the broker will immediately put the
     * message back onto the queue, instead of waiting until the connection is
     * closed. This method is only supported by the RabbitMQ broker. The
     * behavior of calling this method while connected to any other broker is
     * undefined.
     *
     * @param int $deliveryTag Delivery tag of last message to reject.
     * @param int $flags        AMQP_REQUEUE to requeue the message(s),
     *                              AMQP_MULTIPLE to nack all previous
     *                              unacked messages as well.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return bool
     */
    public function nack($deliveryTag, $flags = AMQP_NOPARAM) {}

    /**
     * Mark one message as explicitly not acknowledged.
     *
     * Mark the message identified by delivery_tag as explicitly not
     * acknowledged. This method can only be called on messages that have not
     * yet been acknowledged, meaning that messages retrieved with by
     * AMQPQueue::consume() and AMQPQueue::get() and using the AMQP_AUTOACK
     * flag are not eligible.
     *
     * @param int $deliveryTag Delivery tag of the message to reject.
     * @param int|null $flags        AMQP_REQUEUE to requeue the message(s).
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return bool
     */
    public function reject($deliveryTag, $flags = null) {}

    /**
     * Recover unacknowledged messages delivered to the current consumer.
     *
     * Recover all the unacknowledged messages delivered to the current consumer.
     * If $requeue is true, the broker can redeliver the messages to different
     * consumers. If $requeue is FALSE, it can only redeliver it to the current
     * consumer. RabbitMQ does not implement $request = false.
     * This method exposes `basic.recover` from the AMQP spec.
     *
     * @param bool $requeue If TRUE, deliver to any consumer, if FALSE, deliver to the current consumer only
     * @throws AMQPConnectionException If the connection to the broker was lost.
     * @throws AMQPChannelException If the channel is not open.
     */
    public function recover(bool $requeue = true): void {}

    /**
     * Purge the contents of a queue.
     *
     * Returns the number of purged messages
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return int
     */
    public function purge() {}

    /**
     * Get the argument associated with the given key.
     *
     * @param string $argumentName The key to look up.
     * @throws AMQPQueueException If key does not exist
     * @return bool|int|float|string|null|array|AMQPValue|AMQPDecimal|AMQPTimestamp
     */
    public function getArgument($argumentName) {}

    /**
     * Set a queue argument.
     *
     * @param string $argumentName The key to set.
     * @param bool|int|float|string|null|array|AMQPValue|AMQPDecimal|AMQPTimestamp $argumentValue The argument value to set.
     *
     * @return void
     */
    public function setArgument(string $argumentName, $argumentValue) {}

    /**
     * Set a queue argument.
     *
     * @param string $argumentName The argument name to set.
     */
    public function removeArgument(string $argumentName): void {}

    /**
     * Set all arguments on the given queue.
     *
     * All other argument settings will be wiped.
     *
     * @param array $arguments An array of name/value pairs of arguments.
     */
    public function setArguments(array $arguments): void {}

    /**
     * Get all set arguments as an array of key/value pairs.
     *
     * @return array An array containing all the set key/value pairs.
     */
    public function getArguments(): array {}

    /**
     * Check whether a queue has specific argument.
     *
     * @param string $argumentName The argument name to check.
     *
     * @return bool
     */
    public function hasArgument(string $argumentName): bool {}

    /**
     * Set the flags on the queue.
     *
     * @param int|null $flags A bitmask of flags:
     *                       AMQP_DURABLE, AMQP_PASSIVE,
     *                       AMQP_EXCLUSIVE, AMQP_AUTODELETE.
     *
     * @return bool
     */
    public function setFlags($flags = null) {}

    /**
     * Set the queue name.
     *
     * @param string $name The name of the queue.
     *
     * @return bool
     */
    public function setName($name) {}

    /**
     * Remove a routing key binding on an exchange from the given queue.
     *
     * @param string $exchangeName The name of the exchange on which the
     *                              queue is bound.
     * @param string|null $routingKey   The binding routing key used by the
     *                              queue.
     * @param array  $arguments     Additional binding arguments.
     *
     * @throws AMQPChannelException    If the channel is not open.
     * @throws AMQPConnectionException If the connection to the broker was lost.
     *
     * @return bool
     */
    public function unbind($exchangeName, $routingKey = null, array $arguments = []) {}

    /**
     * Get the AMQPChannel object in use
     *
     * @return AMQPChannel
     */
    public function getChannel() {}

    /**
     * Get the AMQPConnection object in use
     *
     * @return AMQPConnection
     */
    public function getConnection() {}

    /**
     * Get latest consumer tag. If no consumer available or the latest on was canceled null will be returned.
     *
     * @return string|null
     */
    public function getConsumerTag() {}
}

/**
 * stub class representing AMQPQueueException from pecl-amqp
 */
class AMQPQueueException extends AMQPException {}

class AMQPValueException extends AMQPException {}

/**
 * stub class representing AMQPTimestamp from pecl-amqp
 */
final /* readonly */ class AMQPTimestamp implements AMQPValue
{
    public const MIN = 0.0;
    public const MAX = 18446744073709551616;

    /**
     * @throws AMQPValueException
     */
    public function __construct(float $timestamp) {}

    public function __toString(): string {}

    public function getTimestamp(): float {}

    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null {}
}

/**
 * stub class representing AMQPExchangeValue from pecl-amqp
 */
class AMQPExchangeValue extends AMQPException {}
