<?php

class ZMQ
{
    public const SOCKET_PAIR = 0;

    public const SOCKET_PUB = 1;

    public const SOCKET_SUB = 2;

    public const SOCKET_REQ = 3;

    public const SOCKET_REP = 4;

    public const SOCKET_XREQ = 5;

    public const SOCKET_XREP = 6;

    public const SOCKET_PUSH = 8;

    public const SOCKET_PULL = 7;

    public const SOCKET_ROUTER = 6;

    public const SOCKET_DEALER = 5;

    public const SOCKET_XPUB = 9;

    public const SOCKET_XSUB = 10;

    public const SOCKET_STREAM = 11;

    public const SOCKOPT_HWM = 1;

    public const SOCKOPT_SNDHWM = 23;

    public const SOCKOPT_RCVHWM = 24;

    public const SOCKOPT_AFFINITY = 4;

    public const SOCKOPT_IDENTITY = 5;

    public const SOCKOPT_SUBSCRIBE = 6;

    public const SOCKOPT_UNSUBSCRIBE = 7;

    public const SOCKOPT_RATE = 8;

    public const SOCKOPT_RECOVERY_IVL = 9;

    public const SOCKOPT_RECONNECT_IVL = 18;

    public const SOCKOPT_RECONNECT_IVL_MAX = 21;

    public const SOCKOPT_MCAST_LOOP = 10;

    public const SOCKOPT_SNDBUF = 11;

    public const SOCKOPT_RCVBUF = 12;

    public const SOCKOPT_RCVMORE = 13;

    public const SOCKOPT_TYPE = 16;

    public const SOCKOPT_LINGER = 17;

    public const SOCKOPT_BACKLOG = 19;

    public const SOCKOPT_MAXMSGSIZE = 22;

    public const SOCKOPT_SNDTIMEO = 28;

    public const SOCKOPT_RCVTIMEO = 27;

    public const SOCKOPT_IPV4ONLY = 31;

    public const SOCKOPT_LAST_ENDPOINT = 32;

    public const SOCKOPT_TCP_KEEPALIVE_IDLE = 36;

    public const SOCKOPT_TCP_KEEPALIVE_CNT = 35;

    public const SOCKOPT_TCP_KEEPALIVE_INTVL = 37;

    public const SOCKOPT_DELAY_ATTACH_ON_CONNECT = 39;

    public const SOCKOPT_TCP_ACCEPT_FILTER = 38;

    public const SOCKOPT_XPUB_VERBOSE = 40;

    public const SOCKOPT_ROUTER_RAW = 41;

    public const SOCKOPT_IPV6 = 42;

    public const CTXOPT_MAX_SOCKETS = 2;

    public const POLL_IN = 1;

    public const POLL_OUT = 2;

    public const MODE_NOBLOCK = 1;

    public const MODE_DONTWAIT = 1;

    public const MODE_SNDMORE = 2;

    public const DEVICE_FORWARDER = 2;

    public const DEVICE_QUEUE = 3;

    public const DEVICE_STREAMER = 1;

    public const ERR_INTERNAL = -99;

    public const ERR_EAGAIN = 11;

    public const ERR_ENOTSUP = 156384713;

    public const ERR_EFSM = 156384763;

    public const ERR_ETERM = 156384765;

    private function __construct() {}
}

class ZMQContext
{
    /**
     * @param int $io_threads
     * @param bool $is_persistent
     */
    public function __construct($io_threads = 1, $is_persistent = true) {}

    /**
     * @param string $key
     *
     * @return string|int
     *
     * @throws ZMQContextException
     */
    public function getOpt($key)
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Shortcut for creating new sockets from the context.
     * If the context is not persistent the persistent_id parameter is ignored
     * and the socket falls back to being non-persistent.
     * The on_new_socket is called only when a new underlying socket structure is created.
     * @link https://secure.php.net/manual/en/zmqcontext.getsocket.php
     * @param int $type <b>ZMQ::SOCKET_*</b> constant to specify socket type.
     * @param string $persistent_id If persistent_id is specified the socket will be persisted over multiple requests.
     * @param callable $on_new_socket Callback function, which is executed when a new socket structure is created. This function does not get invoked if the underlying persistent connection is re-used. The callback takes ZMQSocket and persistent_id as two arguments.
     * @return ZMQSocket
     * @throws ZMQSocketException
     */
    public function getSocket($type, $persistent_id = null, $on_new_socket = null)
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Whether the context is persistent.
     * Persistent context is needed for persistent connections as each socket is allocated from a context.
     * @link https://secure.php.net/manual/en/zmqcontext.ispersistent.php
     * @return bool Returns <b>TRUE</b> if the context is persistent and <b>FALSE</b> if the context is non-persistent.
     */
    public function isPersistent()
    {
    }

    /**
     * (PECL zmq &gt;= 1.0.4)
     * Sets a ZMQ context option. The type of the value depends on the key.
     * See ZMQ Constant Types for more information.
     * @link https://secure.php.net/manual/en/zmqcontext.setopt.php
     * @param int $key One of the <b>ZMQ::CTXOPT_*<b> constants.
     * @param mixed $value The value of the parameter.
     * @return ZMQContext
     * @throws ZMQContextException
     */
    public function setOpt($key, $value)
    {
    }
}

/**
 * Class ZMQSocket
 * @link https://secure.php.net/manual/en/class.zmqsocket.php
 */
class ZMQSocket
{
    /**
     * (PECL zmq &gt;= 0.5.0)
     * Constructs a ZMQSocket object.
     * The persistent_id parameter can be used to allocated a persistent socket.
     * A persistent socket has to be allocated from a persistent context and it stays connected over multiple requests.
     * The persistent_id parameter can be used to recall the same socket over multiple requests.
     * The on_new_socket is called only when a new underlying socket structure is created.
     * @link https://secure.php.net/manual/en/zmqsocket.construct.php
     * @param ZMQContext $context <p>ZMQContext to build this object</p>
     * @param int $type <p>The type of the socket. See ZMQ::SOCKET_* constants.</p>
     * @param string $persistent_id [optional] <p>If persistent_id is specified the socket will be persisted over multiple requests. If context is not persistent the socket falls back to non-persistent mode.</p>
     * @param callable $on_new_socket [optional] <p>Callback function, which is executed when a new socket structure is created. This function does not get invoked if the underlying persistent connection is re-used.</p>
     * @throws ZMQSocketException
     */
    public function __construct(ZMQContext $context, $type, $persistent_id = null, $on_new_socket = null) {}

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Bind the socket to an endpoint.
     * The endpoint is defined in format transport://address
     * where transport is one of the following: inproc, ipc, tcp, pgm or epgm.
     * @link https://secure.php.net/manual/en/zmqsocket.bind.php
     * @param string $dsn The bind dsn, for example transport://address.
     * @param bool $force Tries to bind even if the socket has already been bound to the given endpoint.
     * @return ZMQSocket
     * @throws ZMQSocketException if binding fails
     */
    public function bind($dsn, $force = false)
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Connect the socket to a remote endpoint.
     * The endpoint is defined in format transport://address
     * where transport is one of the following: inproc, ipc, tcp, pgm or epgm.
     * @link https://secure.php.net/manual/en/zmqsocket.connect.php
     * @param string $dsn The bind dsn, for example transport://address.
     * @param bool $force Tries to bind even if the socket has already been bound to the given endpoint.
     * @return ZMQSocket
     * @throws ZMQSocketException If connection fails
     */
    public function connect($dsn, $force = false)
    {
    }

    /**
     * (PECL zmq &gt;= 1.0.4)
     * Disconnect the socket from a previously connected remote endpoint.
     * The endpoint is defined in format transport://address
     * where transport is one of the following: inproc, ipc, tcp, pgm or epgm.
     * @link https://secure.php.net/manual/en/zmqsocket.disconnect.php
     * @param string $dsn The bind dsn, for example transport://address.
     * @return ZMQSocket
     * @throws ZMQSocketException If connection fails
     */
    public function disconnect($dsn)
    {
    }

    /**
     * Returns a list of endpoints where the socket is connected or bound to.
     * @link https://secure.php.net/manual/en/zmqsocket.getendpoints.php
     * @return array contains two sub-arrays: 'connect' and 'bind'
     * @throws ZMQSocketException
     */
    public function getEndpoints()
    {
    }

    /**
     * Returns the persistent id string assigned of the object and NULL if socket is not persistent.
     * @link https://secure.php.net/manual/en/zmqsocket.getpersistentid.php
     * @return string|null <p>
     * Returns the persistent id string assigned of the object and <b>NULL</b> if socket is not persistent.
     * </p>
     */
    public function getPersistentId()
    {
    }

    /**
     * Returns the value of a socket option.
     * This method is available if ZMQ extension has been compiled against ZMQ version 2.0.7 or higher
     * @link https://secure.php.net/manual/en/zmqsocket.getsockopt.php
     * @since 0MQ 2.0.7
     * @param int $key An int representing the option. See the <b>ZMQ::SOCKOPT_*</b> constants.
     * @return string|int <p>
     * Returns either a string or an integer depending on <b>key</b>. Throws
     * ZMQSocketException on error.
     * </p>
     * @throws ZMQSocketException
     */
    public function getSockOpt($key)
    {
    }

    /**
     * Return the socket type.
     * The socket type can be compared against ZMQ::SOCKET_* constants.
     * @link https://secure.php.net/manual/en/zmqsocket.getsockettype.php
     * @return int <p>
     * Returns an integer representing the socket type. The integer can be compared against
     * <b>ZMQ::SOCKET_*</b> constants.
     * </p>
     */
    public function getSocketType()
    {
    }

    /**
     * Check whether the socket is persistent.
     * @link https://secure.php.net/manual/en/zmqsocket.ispersistent.php
     * @return bool <p>Returns a boolean based on whether the socket is persistent or not.</p>
     */
    public function isPersistent()
    {
    }

    /**
     * Receive a message from a socket.
     * By default receiving will block until a message is available unless <b>ZMQ::MODE_NOBLOCK</b> flag is used.
     * <b>ZMQ::SOCKOPT_RCVMORE</b> socket option can be used for receiving multi-part messages.
     * Returns the message.
     * If <b>ZMQ::MODE_NOBLOCK</b> is used and the operation would block bool false shall be returned.
     * @link https://secure.php.net/manual/en/zmqsocket.recv.php
     * @see ZMQSocket::setSockOpt()
     * @param int $mode Pass mode flags to receive multipart messages or non-blocking operation. See ZMQ::MODE_* constants.
     * @return string|false <p>Returns the message. Throws ZMQSocketException in error. If <b>ZMQ::MODE_NOBLOCK</b> is used and the operation would block boolean false shall be returned.</p>
     * @throws ZMQSocketException if receiving fails.
     */
    public function recv($mode = 0)
    {
    }

    /**
     * Receive an array multipart message from a socket.
     * By default receiving will block until a message is available unless ZMQ::MODE_NOBLOCK flag is used.
     * Returns the array of message parts.
     * If <b>ZMQ::MODE_NOBLOCK</b> is used and the operation would block bool false shall be returned.
     * @link https://secure.php.net/manual/en/zmqsocket.recvmulti.php
     * @param int $mode Pass mode flags to receive multipart messages or non-blocking operation. See ZMQ::MODE_* constants.
     * @return string[] Returns the array of message parts. Throws ZMQSocketException in error. If ZMQ::MODE_NOBLOCK is used and the operation would block boolean false shall be returned.
     * @throws ZMQSocketException if receiving fails.
     */
    public function recvMulti($mode = 0)
    {
    }

    /**
     * Send a message using the socket. The operation can block unless ZMQ::MODE_NOBLOCK is used.
     * If <b>ZMQ::MODE_NOBLOCK</b> is used and the operation would block bool false shall be returned.
     * @link https://secure.php.net/manual/en/zmqsocket.send.php
     * @param string $message The message to send
     * @param int $mode Pass mode flags to receive multipart messages or non-blocking operation. See ZMQ::MODE_* constants.     *
     * @return ZMQSocket
     * @throws ZMQSocketException if sending message fails
     */
    public function send($message, $mode = 0)
    {
    }

    /**
     * Send a multipart message using the socket. The operation can block unless ZMQ::MODE_NOBLOCK is used.
     * If <b>ZMQ::MODE_NOBLOCK</b> is used and the operation would block bool false shall be returned.
     * @link https://secure.php.net/manual/en/zmqsocket.sendmulti.php
     * @param array $message The message to send - an array of strings
     * @param int $mode Pass mode flags to receive multipart messages or non-blocking operation. See ZMQ::MODE_* constants.     *
     * @return ZMQSocket
     * @throws ZMQSocketException if sending message fails
     */
    public function sendmulti(array $message, $mode = 0)
    {
    }

    /**
     * Sets a ZMQ socket option. The type of the value depends on the key.
     * @param int $key One of the <b>ZMQ::SOCKOPT_*</b> constants.
     * @param mixed $value The value of the parameter.
     * @return ZMQSocket
     * @throws ZMQSocketException
     * @see ZMQ Constant Types for more information.
     * @link https://secure.php.net/manual/en/zmqsocket.setsockopt.php
     */
    public function setSockOpt($key, $value)
    {
    }

    /**
     * Unbind the socket from an endpoint.
     * The endpoint is defined in format transport://address
     * where transport is one of the following: inproc, ipc, tcp, pgm or epgm.
     * @link https://secure.php.net/manual/en/zmqsocket.unbind.php
     * @param string $dsn The previously bound dsn, for example transport://address.
     * @return ZMQSocket
     * @throws ZMQSocketException if binding fails
     */
    public function unbind($dsn)
    {
    }
}

/**
 * Class ZMQPoll
 * @link https://secure.php.net/manual/en/class.zmqpoll.php
 */
class ZMQPoll
{
    /**
     * (PECL zmq &gt;= 0.5.0)
     * Adds a new item to the poll set and returns the internal id of the added item.
     * The item can be removed from the poll set using the returned string id.
     * Returns a string id of the added item which can be later used to remove the item.
     * @link https://secure.php.net/manual/en/zmqpoll.add.php
     * @param ZMQSocket $entry ZMQSocket object or a PHP stream resource
     * @param int $type Defines what activity the socket is polled for. See <b>ZMQ::POLL_IN</b> and <b>ZMQ::POLL_OUT</b> constants.
     * @return int Returns a string id of the added item which can be later used to remove the item. Throws ZMQPollException on error.
     * @throws ZMQPollException if the object has not been initialized with polling
     */
    public function add(ZMQSocket $entry, $type)
    {
    }

    /**
     * (PECL zmq &gt;= 1.0.4)
     * Clears all elements from the poll set.
     * @link https://secure.php.net/manual/en/zmqpoll.clear.php
     * @return ZMQPoll Returns the current object.
     */
    public function clear()
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Count the items in the poll set.
     * @link https://secure.php.net/manual/en/zmqpoll.count.php
     * @return int Returns an integer representing the amount of items in the poll set.
     */
    public function count()
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Returns the ids of the objects that had errors in the last poll.
     * Returns an array containing ids for the items that had errors in the last poll.
     * Empty array is returned if there were no errors.
     * @link https://secure.php.net/manual/en/zmqpoll.getlasterrors.php
     * @return int[]
     */
    public function getLastErrors()
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Polls the items in the current poll set.
     * The readable and writable items are returned in the readable and writable parameters.
     * ZMQPoll::getLastErrors() can be used to check if there were errors.
     * Returns an int representing amount of items with activity.
     * @link https://secure.php.net/manual/en/zmqpoll.poll.php
     * @param array &$readable Array where readable ZMQSockets/PHP streams are returned. The array will be cleared at the beginning of the operation.
     * @param array &$writable Array where writable ZMQSockets/PHP streams are returned. The array will be cleared at the beginning of the operation.
     * @param int $timeout Timeout for the operation. -1 means that poll waits until at least one item has activity. Please note that starting from version 1.0.0 the poll timeout is defined in milliseconds, rather than microseconds.
     * @throws ZMQPollException if polling fails
     * @return int
     */
    public function poll(array &$readable, array &$writable, $timeout = -1)
    {
    }

    /**
     * (PECL zmq &gt;= 0.5.0)
     * Remove item from the poll set.
     * The item parameter can be ZMQSocket object, a stream resource or the id returned from ZMQPoll::add() method.
     * Returns true if the item was removed and false if the object with given id does not exist in the poll set.
     * @link https://secure.php.net/manual/en/zmqpoll.remove.php
     * @param ZMQSocket|string|mixed $item The ZMQSocket object, PHP stream or string id of the item.
     * @return bool Returns true if the item was removed and false if the object with given id does not exist in the poll set.
     */
    public function remove($item)
    {
    }
}

/**
 * Class ZMQDevice
 * @link https://secure.php.net/manual/en/class.zmqdevice.php
 */
class ZMQDevice
{
    /**
     * (PECL zmq &gt;= 1.0.4)
     * Construct a new device.
     * "ØMQ devices can do intermediation of addresses, services, queues, or any other abstraction you care
     * to define above the message and socket layers." -- zguide
     * Call to this method will prepare the device. Usually devices are very long running processes so running this method from interactive script is not recommended. This method throw ZMQDeviceException if the device cannot be started.
     * @link https://secure.php.net/manual/en/zmqdevice.construct.php
     * @param ZMQSocket $frontend Frontend parameter for the devices. Usually where there messages are coming.
     * @param ZMQSocket $backend Backend parameter for the devices. Usually where there messages going to.
     * @param null|ZMQSocket $listener Listener socket, which receives a copy of all messages going both directions. The type of this socket should be SUB, PULL or DEALER.
     */
    public function __construct(ZMQSocket $frontend, ZMQSocket $backend, ZMQSocket $listener = null) {}

    /**
     * Gets the idle callback timeout value.
     * This method returns the idle callback timeout value.
     * Added in ZMQ extension version 1.1.0.
     * @link https://secure.php.net/manual/en/zmqdevice.getidletimeout.php
     * @return int This method returns the idle callback timeout value.
     */
    public function getIdleTimeout()
    {
    }

    /**
     * Gets the timer callback timeout value.
     * Added in ZMQ extension version 1.1.0.
     * @link https://secure.php.net/manual/en/zmqdevice.gettimertimeout.php
     * @return int This method returns the timer timeout value.
     */
    public function getTimerTimeout()
    {
    }

    /**
     * Runs the device.
     * Call to this method will block until the device is running.
     * It is not recommended that devices are used from interactive scripts.
     * @link https://secure.php.net/manual/en/zmqdevice.run.php
     * @throws ZMQDeviceException
     */
    public function run()
    {
    }

    /**
     * Sets the idle callback function.
     * If idle timeout is defined the idle callback function shall be called if the internal poll loop times out
     * without events. If the callback function returns false or a value that evaluates to false the device is stopped.
     * The callback function signature is callback (mixed $user_data).
     * @link https://secure.php.net/manual/en/zmqdevice.setidlecallback.php
     * @param callable $cb_func Callback function to invoke when the device is idle. Returning false or a value that evaluates to false from this function will cause the device to stop.
     * @param int $timeout How often to invoke the idle callback in milliseconds. The idle callback is invoked periodically when there is no activity on the device. The timeout value guarantees that there is at least this amount of milliseconds between invocations of the callback function.
     * @param mixed $user_data Additional data to pass to the callback function.
     * @return ZMQDevice On success this method returns the current object.
     */
    public function setIdleCallback($cb_func, $timeout, $user_data)
    {
    }

    /**
     * Sets the idle callback timeout value. The idle callback is invoked periodically when the device is idle.
     * On success this method returns the current object.
     * @link https://secure.php.net/manual/en/zmqdevice.setidletimeout.php
     * @param int $timeout The idle callback timeout value in milliseconds
     * @return ZMQDevice On success this method returns the current object.
     */
    public function setIdleTimeout($timeout)
    {
    }

    /**
     * @param callable $cb_func
     * @param int $timeout
     * @param mixed $user_data
     *
     * @return ZMQDevice
     */
    public function setTimerCallback($cb_func, $timeout, $user_data)
    {
    }

    /**
     * @param int $timeout
     *
     * @return ZMQDevice
     */
    public function setTimerTimeout($timeout)
    {
    }
}

class ZMQException extends Exception
{
}

class ZMQContextException extends ZMQException
{
}

class ZMQSocketException extends ZMQException
{
}

class ZMQPollException extends ZMQException
{
}

class ZMQDeviceException extends ZMQException
{
}
