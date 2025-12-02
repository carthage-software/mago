<?php

const AMQP_NOPARAM = 0;

const AMQP_JUST_CONSUME = 1;

const AMQP_DURABLE = 2;

const AMQP_PASSIVE = 4;

const AMQP_EXCLUSIVE = 8;

const AMQP_AUTODELETE = 16;

const AMQP_INTERNAL = 32;

const AMQP_NOLOCAL = 64;

const AMQP_AUTOACK = 128;

const AMQP_IFEMPTY = 256;

const AMQP_IFUNUSED = 512;

const AMQP_MANDATORY = 1024;

const AMQP_IMMEDIATE = 2048;

const AMQP_MULTIPLE = 4096;

const AMQP_NOWAIT = 8192;

const AMQP_REQUEUE = 16384;

const AMQP_EX_TYPE_DIRECT = 'direct';

const AMQP_EX_TYPE_FANOUT = 'fanout';

const AMQP_EX_TYPE_TOPIC = 'topic';

const AMQP_EX_TYPE_HEADERS = 'headers';

const AMQP_OS_SOCKET_TIMEOUT_ERRNO = 536870947;

const PHP_AMQP_MAX_CHANNELS = 256;

const AMQP_SASL_METHOD_PLAIN = 0;

const AMQP_SASL_METHOD_EXTERNAL = 1;

const AMQP_DELIVERY_MODE_TRANSIENT = 1;

const AMQP_DELIVERY_MODE_PERSISTENT = 2;

const AMQP_EXTENSION_VERSION = '1.1.12alpha3';

const AMQP_EXTENSION_VERSION_MAJOR = 0;

const AMQP_EXTENSION_VERSION_MINOR = 1;

const AMQP_EXTENSION_VERSION_PATCH = 12;

const AMQP_EXTENSION_VERSION_EXTRA = 'alpha3';

const AMQP_EXTENSION_VERSION_ID = '10112';

class AMQPBasicProperties
{
    public function __construct(
        null|string $contentType = null,
        null|string $contentEncoding = null,
        array $headers = [],
        int $deliveryMode = AMQP_DELIVERY_MODE_TRANSIENT,
        int $priority = 0,
        null|string $correlationId = null,
        null|string $replyTo = null,
        null|string $expiration = null,
        null|string $messageId = null,
        null|int $timestamp = null,
        null|string $type = null,
        null|string $userId = null,
        null|string $appId = null,
        null|string $clusterId = null,
    ) {}

    /**
     * @return string|null
     */
    public function getContentType()
    {
    }

    /**
     * @return string|null
     */
    public function getContentEncoding()
    {
    }

    /**
     * @return array
     */
    public function getHeaders()
    {
    }

    /**
     * @return int
     */
    public function getDeliveryMode()
    {
    }

    /**
     * @return int
     */
    public function getPriority()
    {
    }

    /**
     * @return string|null
     */
    public function getCorrelationId()
    {
    }

    /**
     * @return string|null
     */
    public function getReplyTo()
    {
    }

    /**
     * @return string|null
     */
    public function getExpiration()
    {
    }

    /**
     * @return string|null
     */
    public function getMessageId()
    {
    }

    /**
     * @return int|null
     */
    public function getTimestamp()
    {
    }

    /**
     * @return string|null
     */
    public function getType()
    {
    }

    /**
     * @return string|null
     */
    public function getUserId()
    {
    }

    /**
     * @return string|null
     */
    public function getAppId()
    {
    }

    /**
     * @return string|null
     */
    public function getClusterId()
    {
    }
}

class AMQPChannel
{
    /**
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function commitTransaction()
    {
    }

    /**
     * @param AMQPConnection $amqp_connection
     *
     * @throws AMQPConnectionException
     */
    public function __construct(AMQPConnection $amqp_connection) {}

    /**
     * @return bool
     */
    public function isConnected()
    {
    }

    /**
     * @return void
     */
    public function close()
    {
    }

    /**
     * @return int
     */
    public function getChannelId()
    {
    }

    /**
     * @param int $size
     * @param int $count
     * @param bool $global
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function qos($size, $count, $global = false)
    {
    }

    /**
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function rollbackTransaction()
    {
    }

    /**
     * @param int $count
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setPrefetchCount($count)
    {
    }

    /**
     * @return int
     */
    public function getPrefetchCount()
    {
    }

    /**
     * @param int $size
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setPrefetchSize($size)
    {
    }

    /**
     * @return int
     */
    public function getPrefetchSize()
    {
    }

    /**
     * @param int $count
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setGlobalPrefetchCount($count)
    {
    }

    /**
     * @return int
     */
    public function getGlobalPrefetchCount()
    {
    }

    /**
     * @param int $size
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setGlobalPrefetchSize($size)
    {
    }

    /**
     * @return int
     */
    public function getGlobalPrefetchSize()
    {
    }

    /**
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function startTransaction()
    {
    }

    /**
     * @return AMQPConnection
     */
    public function getConnection()
    {
    }

    /**
     * @param bool $requeue
     *
     * @return void
     */
    public function basicRecover($requeue = true)
    {
    }

    /**
     * @return void
     */
    public function confirmSelect()
    {
    }

    /**
     * @param callable|null $ack_callback
     * @param callable|null $nack_callback
     *
     * @return void
     */
    public function setConfirmCallback(callable $ack_callback = null, callable $nack_callback = null)
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPQueueException
     *
     * @return void
     */
    public function waitForConfirm($timeout = 0.0)
    {
    }

    /**
     * @param callable|null $return_callback
     *
     * @return void
     */
    public function setReturnCallback(callable $return_callback = null)
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPQueueException
     *
     * @return void
     */
    public function waitForBasicReturn($timeout = 0.0)
    {
    }

    /**
     * @return AMQPQueue[]
     */
    public function getConsumers()
    {
    }
}

class AMQPChannelException extends AMQPException
{
}

class AMQPConnection
{
    /**
     * @param array $credentials
     */
    public function __construct(array $credentials = []) {}

    /**
     * @return bool
     */
    public function isConnected()
    {
    }

    /**
     * @return bool
     */
    public function isPersistent()
    {
    }

    /**
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function connect()
    {
    }

    /**
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function disconnect()
    {
    }

    /**
     * @return void
     */
    public function reconnect()
    {
    }

    /**
     * @throws AMQPConnectionException
     * @return void
     */
    public function pconnect()
    {
    }

    /**
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function pdisconnect()
    {
    }

    /**
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function preconnect()
    {
    }

    /**
     * @return string
     */
    public function getHost()
    {
    }

    /**
     * @return string
     */
    public function getLogin()
    {
    }

    /**
     * @return string
     */
    public function getPassword()
    {
    }

    /**
     * @return int
     */
    public function getPort()
    {
    }

    /**
     * @return string
     */
    public function getVhost()
    {
    }

    /**
     * @param string $host
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setHost($host)
    {
    }

    /**
     * @param string $login
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setLogin($login)
    {
    }

    /**
     * @param string $password
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setPassword($password)
    {
    }

    /**
     * @param int $port
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setPort($port)
    {
    }

    /**
     * @param string $vhost
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setVhost($vhost)
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    #[Deprecated]
    public function setTimeout($timeout)
    {
    }

    /**
     * @return float
     */
    #[Deprecated]
    public function getTimeout()
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setReadTimeout($timeout)
    {
    }

    /**
     * @return float
     */
    public function getReadTimeout()
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setWriteTimeout($timeout)
    {
    }

    /**
     * @return float
     */
    public function getWriteTimeout()
    {
    }

    /**
     */
    public function getConnectTimeout(): float
    {
    }

    /**
     * @param float $timeout
     *
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function setRpcTimeout($timeout)
    {
    }

    /**
     * @return float
     */
    public function getRpcTimeout()
    {
    }

    /**
     * @return int
     */
    public function getUsedChannels()
    {
    }

    /**
     * @return int
     */
    public function getMaxChannels()
    {
    }

    /**
     * @return int
     */
    public function getMaxFrameSize()
    {
    }

    /**
     * @return int
     */
    public function getHeartbeatInterval()
    {
    }

    /**
     * @return string|null
     */
    public function getCACert()
    {
    }

    /**
     * @param string $cacert
     *
     * @return void
     */
    public function setCACert($cacert)
    {
    }

    /**
     * @return string|null
     */
    public function getCert()
    {
    }

    /**
     * @param string $cert
     *
     * @return void
     */
    public function setCert($cert)
    {
    }

    /**
     * @return string|null
     */
    public function getKey()
    {
    }

    /**
     * @param string|null $key
     *
     * @return void
     */
    public function setKey($key)
    {
    }

    /**
     * @return bool
     */
    public function getVerify()
    {
    }

    /**
     * @param bool $verify
     *
     * @return void
     */
    public function setVerify($verify)
    {
    }

    /**
     * @param int $method
     *
     * @return void
     */
    public function setSaslMethod($method)
    {
    }

    /**
     * @return int
     */
    public function getSaslMethod()
    {
    }

    public function setConnectionName(null|string $connectionName): void
    {
    }

    public function getConnectionName(): null|string
    {
    }
}

class AMQPConnectionException extends AMQPException
{
}

interface AMQPValue
{
    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null;
}

final class AMQPDecimal implements AMQPValue
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
    public function getExponent()
    {
    }

    /** @return int */
    public function getSignificand()
    {
    }

    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null
    {
    }
}

class AMQPEnvelope extends AMQPBasicProperties
{
    /**
     * @return string
     */
    public function getBody()
    {
    }

    /**
     * @return string
     */
    public function getRoutingKey()
    {
    }

    /**
     * @return string|null
     */
    public function getConsumerTag()
    {
    }

    /**
     * @return int|null
     */
    public function getDeliveryTag()
    {
    }

    /**
     * @return string|null
     */
    public function getExchangeName()
    {
    }

    /**
     * @return bool
     */
    public function isRedelivery()
    {
    }

    /**
     * @param string $headerName
     */
    public function getHeader($headerName)
    {
    }

    /**
     * @param string $headerName
     *
     * @return bool
     */
    public function hasHeader($headerName)
    {
    }
}

class AMQPEnvelopeException extends AMQPException
{
    public function getEnvelope(): AMQPEnvelope
    {
    }
}

class AMQPException extends Exception
{
}

class AMQPExchange
{
    /**
     * @param AMQPChannel $channel
     *
     * @throws AMQPExchangeException
     * @throws AMQPConnectionException
     */
    public function __construct(AMQPChannel $channel) {}

    /**
     * @param string $exchangeName
     * @param string $routingKey
     * @param array  $arguments
     *
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function bind($exchangeName, $routingKey = '', array $arguments = [])
    {
    }

    /**
     * @param string $exchangeName
     * @param string $routingKey
     * @param array  $arguments
     *
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function unbind($exchangeName, $routingKey = '', array $arguments = [])
    {
    }

    /**
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function declareExchange()
    {
    }

    /**
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function declare(): void
    {
    }

    /**
     * @param string  $exchangeName
     * @param int $flags
     *
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function delete($exchangeName = null, $flags = AMQP_NOPARAM)
    {
    }

    /**
     * @param string $argumentName
     *
     * @throws AMQPExchangeException
     *
     * @return bool|int|float|string|null
     */
    public function getArgument($argumentName)
    {
    }

    /**
     * @param string $argumentName
     *
     * @return bool
     */
    public function hasArgument($argumentName)
    {
    }

    /**
     * @return array
     */
    public function getArguments()
    {
    }

    /**
     * @return int
     */
    public function getFlags()
    {
    }

    /**
     * @return string|null
     */
    public function getName()
    {
    }

    /**
     * @return string|null
     */
    public function getType()
    {
    }

    /**
     * @param string $message
     * @param string|null $routingKey
     * @param int|null $flags
     * @param array $headers
     *
     * @throws AMQPExchangeException
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function publish($message, $routingKey = null, $flags = null, array $headers = [])
    {
    }

    /**
     * @param string $argumentName
     * @param string|int $argumentValue
     *
     * @return void
     */
    public function setArgument($argumentName, $argumentValue)
    {
    }

    /**
     * @param string $argumentName
     */
    public function removeArgument(string $argumentName): void
    {
    }

    /**
     * @return bool
     */
    public function setArguments(array $arguments)
    {
    }

    /**
     * @param int|null $flags
     *
     * @return void
     */
    public function setFlags($flags)
    {
    }

    /**
     * @param string $exchangeName
     *
     * @return void
     */
    public function setName($exchangeName)
    {
    }

    /**
     * @param string $exchangeType
     *
     * @return void
     */
    public function setType($exchangeType)
    {
    }

    /**
     * @return AMQPChannel
     */
    public function getChannel()
    {
    }

    /**
     * @return AMQPConnection
     */
    public function getConnection()
    {
    }
}

class AMQPExchangeException extends AMQPException
{
}

class AMQPQueue
{
    /**
     * @throws AMQPQueueException
     * @throws AMQPConnectionException
     */
    public function __construct(AMQPChannel $channel) {}

    /**
     * @param int $deliveryTag
     * @param int|null $flags
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return void
     */
    public function ack($deliveryTag, $flags = null)
    {
    }

    /**
     * @param string $exchangeName
     * @param string|null $routingKey
     * @param array  $arguments
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return bool
     */
    public function bind($exchangeName, $routingKey = null, array $arguments = [])
    {
    }

    /**
     * @param string $consumer_tag
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return bool
     */
    public function cancel($consumer_tag = '')
    {
    }

    /**
     * @param int|null $flags
     * @param string|null $consumerTag
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     * @throws AMQPEnvelopeException
     * @throws AMQPQueueException
     *
     * @return void
     */
    public function consume(callable $callback = null, $flags = null, $consumerTag = null)
    {
    }

    /**
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     * @throws AMQPQueueException
     *
     * @return int
     */
    public function declareQueue()
    {
    }

    /**
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     * @throws AMQPQueueException
     *
     * @return int
     */
    public function declare(): int
    {
    }

    /**
     * @param int $flags
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return int
     */
    public function delete($flags = AMQP_NOPARAM)
    {
    }

    /**
     * @param null|int $flags
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     * @throws AMQPQueueException
     *
     * @return AMQPEnvelope|null
     */
    public function get($flags = null)
    {
    }

    public function getFlags(): int
    {
    }

    public function getName(): null|string
    {
    }

    /**
     * @param int $deliveryTag
     * @param int $flags
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return bool
     */
    public function nack($deliveryTag, $flags = AMQP_NOPARAM)
    {
    }

    /**
     * @param int $deliveryTag
     * @param int|null $flags
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return bool
     */
    public function reject($deliveryTag, $flags = null)
    {
    }

    /**
     * @throws AMQPConnectionException
     * @throws AMQPChannelException
     */
    public function recover(bool $requeue = true): void
    {
    }

    /**
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     *
     * @return int
     */
    public function purge()
    {
    }

    /**
     * @param string $argumentName
     *
     * @return bool|int|float|string|null|array|AMQPValue|AMQPDecimal|AMQPTimestamp
     *
     * @throws AMQPQueueException
     */
    public function getArgument($argumentName)
    {
    }

    /**
     * @param bool|int|float|string|null|array|AMQPValue|AMQPDecimal|AMQPTimestamp $argumentValue The argument value to set.
     *
     * @return void
     */
    public function setArgument(string $argumentName, $argumentValue)
    {
    }

    public function removeArgument(string $argumentName): void
    {
    }

    public function setArguments(array $arguments): void
    {
    }

    public function getArguments(): array
    {
    }

    public function hasArgument(string $argumentName): bool
    {
    }

    /**
     * @param int|null $flags
     *
     * @return bool
     */
    public function setFlags($flags = null)
    {
    }

    /**
     * @param string $name
     *
     * @return bool
     */
    public function setName($name)
    {
    }

    /**
     * @param string $exchangeName
     * @param string|null $routingKey
     *
     * @return bool
     *
     * @throws AMQPChannelException
     * @throws AMQPConnectionException
     */
    public function unbind($exchangeName, $routingKey = null, array $arguments = [])
    {
    }

    /**
     * @return AMQPChannel
     */
    public function getChannel()
    {
    }

    /**
     * @return AMQPConnection
     */
    public function getConnection()
    {
    }

    /**
     * @return string|null
     */
    public function getConsumerTag()
    {
    }
}

class AMQPQueueException extends AMQPException
{
}

class AMQPValueException extends AMQPException
{
}

final class AMQPTimestamp implements AMQPValue
{
    public const MIN = 0.0;
    public const MAX = 18446744073709551616;

    /**
     * @throws AMQPValueException
     */
    public function __construct(float $timestamp) {}

    public function __toString(): string
    {
    }

    public function getTimestamp(): float
    {
    }

    public function toAmqpValue(): float|array|AMQPDecimal|bool|int|AMQPValue|string|AMQPTimestamp|null
    {
    }
}

class AMQPExchangeValue extends AMQPException
{
}
