<?php

const YAR_VERSION = '2.2.0';

const YAR_CLIENT_PROTOCOL_HTTP = 1;

const YAR_OPT_PACKAGER = 1;

const YAR_OPT_TIMEOUT = 4;

const YAR_OPT_CONNECT_TIMEOUT = 8;

const YAR_OPT_PERSISTENT = 2;

const YAR_OPT_HEADER = 16;

const YAR_PACKAGER_PHP = 'PHP';

const YAR_PACKAGER_JSON = 'JSON';

const YAR_ERR_OUTPUT = 8;

const YAR_ERR_OKEY = 0;

const YAR_ERR_TRANSPORT = 16;

const YAR_ERR_REQUEST = 4;

const YAR_ERR_PROTOCOL = 2;

const YAR_ERR_PACKAGER = 1;

const YAR_ERR_EXCEPTION = 64;

const YAR_CLIENT_PROTOCOL_TCP = 2;

const YAR_CLIENT_PROTOCOL_UNIX = 4;

const YAR_OPT_RESOLVE = 32;

class Yar_Server
{
    protected $_executor;

    /**
     * @param object $obj
     */
    final public function __construct($obj, $protocol = null) {}

    /**
     * @return bool
     */
    public function handle() {}
}

class Yar_Client
{
    protected $_protocol;
    protected $_uri;
    protected $_options;
    protected $_running;

    /**
     * @param string $method
     * @param array $parameters
     */
    public function __call($method, $parameters) {}

    /**
     * @param string $url
     */
    final public function __construct($url, $async = null) {}

    public function call($method, $parameters) {}

    /**
     * @param int $type
     * @param $value
     *
     * @return static|false
     */
    public function setOpt($type, $value) {}

    public function getOpt($type) {}
}

class Yar_Concurrent_Client
{
    protected static $_callstack;
    protected static $_callback;
    protected static $_error_callback;
    protected static $_start;

    /**
     * @param string $uri
     * @param string $method
     * @param array $parameters
     * @param callable $callback
     * @param callable $error_callback
     * @param array $options
     *
     * @return int
     */
    public static function call(
        $uri,
        $method,
        $parameters,
        callable $callback = null,
        callable $error_callback,
        array $options,
    ) {}

    /**
     * @param callable $callback
     * @param callable $error_callback
     *
     * @return bool
     */
    public static function loop($callback = null, $error_callback = null) {}

    /**
     * @return bool
     */
    public static function reset() {}
}

class Yar_Server_Exception extends Exception
{
    protected $_type;

    /**
     * @return string
     */
    public function getType() {}
}

class Yar_Client_Exception extends Exception
{
    /**
     * @return string
     */
    public function getType() {}
}

class Yar_Server_Request_Exception extends Yar_Server_Exception {}

class Yar_Server_Protocol_Exception extends Yar_Server_Exception {}

class Yar_Server_Packager_Exception extends Yar_Server_Exception {}

class Yar_Server_Output_Exception extends Yar_Server_Exception {}

class Yar_Client_Transport_Exception extends Yar_Client_Exception {}

class Yar_Client_Packager_Exception extends Yar_Client_Exception {}

class Yar_Client_Protocol_Exception extends Yar_Client_Exception {}
